use std::{thread::spawn, time::Duration};

use cataklysm::{game::*, pair::Pair};

use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select,
    sync::mpsc::{error::TryRecvError, unbounded_channel},
};

enum TeiMsg {
    Game(Box<dyn Game>),
    Play(String),
    Search {
        time: Pair<Duration>,
        increment: Pair<Duration>,
    },
}

enum WorkerMsg {
    Best(Box<dyn Action>),
}

pub async fn run() {
    use TeiMsg::*;
    use WorkerMsg::*;

    let mut lines = BufReader::new(stdin()).lines();

    assert_eq!(lines.next_line().await.unwrap().unwrap(), "tei");
    println!("id name cataklysm");
    println!("id author alion02");
    println!("teiok");

    let send = unbounded_channel::<TeiMsg>();
    let recv = unbounded_channel::<WorkerMsg>();
    spawn(move || {
        let mut rx = send.1;
        let tx = recv.0;

        let Some(Game(mut game)) = rx.blocking_recv() else {
            panic!()
        };

        // Simplify control logic for search thread. Move the Game between the threads instead of
        // making a convoluted message-passing mess. The search thread should only do the following:
        // - check an abort flag
        // - manage time controls
        let mut msg;
        'outer: loop {
            {
                let Some(next_msg) = rx.blocking_recv() else {
                    break;
                };
                msg = next_msg;
            }

            'idle: loop {
                match msg {
                    Game(new_game) => game = new_game,
                    Play(s) => {
                        let action = game.parse_action(&s).unwrap();
                        game.play(action).unwrap();
                    }
                    Search { time, increment } => {
                        let mut d = 1;
                        loop {
                            match rx.try_recv() {
                                Err(TryRecvError::Empty) => (),
                                Err(TryRecvError::Disconnected) => break 'outer,
                                Ok(next_msg) => {
                                    msg = next_msg;
                                    continue 'idle;
                                }
                            }

                            game.search(d);
                            d += 1;
                        }
                    }
                }

                break;
            }
        }
    });

    let rx = recv.1;
    let tx = send.0;

    let mut history = vec![];

    loop {
        select! {
            biased;

            line = lines.next_line() => {
                let line = line.unwrap().unwrap();
                let mut cmd = line.split_ascii_whitespace();
                match cmd.next().unwrap() {
                    "isready" => println!("readyok"),
                    "teinewgame" => {
                        let size = cmd.next().unwrap().parse().unwrap();
                        let game = new_game(size, Options::default(size).unwrap()).unwrap();
                        tx.send(Game(game)).unwrap();
                    }
                    "position" => {
                        assert_eq!(cmd.next().unwrap(), "startpos");
                        assert_eq!(cmd.next().unwrap(), "moves");

                        assert!(
                            history.iter().zip(cmd.by_ref()).all(|(curr, new)| curr == new),
                            "undo not yet supported",
                        );

                        for m in cmd {
                            // TODO: Sincere apologies
                            tx.send(Play(m.to_string())).unwrap();
                            history.push(m.to_string());
                        }
                    }
                    "go" => {
                        let time = Pair::default();
                        let increment = Pair::default();

                        while let Some(subcmd) = cmd.next() {
                            let target = &mut match subcmd {
                                "wtime" => time.white,
                                "btime" => time.black,
                                "winc" => increment.white,
                                "binc" => increment.black,
                                _ => panic!(r#"unsupported command "{line}" @ "{subcmd}""#),
                            };

                            *target = Duration::from_millis(cmd.next().unwrap().parse().unwrap());
                        }

                        tx.send(Search { time, increment }).unwrap();
                    }

                    _ => panic!(r#"unsupported command "{line}""#),
                }
            }
        }
    }
}

use std::{thread::spawn, time::Duration};

use cataklysm::{game::*, pair::Pair};

use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select,
    sync::mpsc::{error::TryRecvError, unbounded_channel},
};

enum TeiMsg {
    Game(Box<dyn Game>),
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

    loop {
        let line = lines.next_line().await.unwrap().unwrap();
        let mut cmd = line.split_ascii_whitespace();
        match cmd.next().unwrap() {
            "isready" => println!("readyok"),
            "teinewgame" => {
                let size = cmd.next().unwrap().parse().unwrap();
                let game = new_game(size, Options::default(size).unwrap()).unwrap();
            }

            _ => println!(r#"info string received unknown command "{line}""#),
        }
    }
}

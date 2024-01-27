use std::{
    thread::spawn,
    time::{Duration, Instant},
};

use cataklysm::{game::*, pair::Pair};

use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};

struct State {
    rx: UnboundedReceiver<Box<dyn Game>>,
    tx: UnboundedSender<Search>,
    history: Vec<String>, // TODO: Use Box<dyn Action>?
    game: Option<Box<dyn Game>>,
    flag: Option<AbortFlag>,
}

impl State {
    async fn abort(&mut self) {
        if let Some(flag) = self.flag.take() {
            flag.set();
            self.game = Some(self.rx.recv().await.unwrap());
        }
    }
}

struct Search {
    game: Box<dyn Game>,
    time_target: Duration,
}

pub async fn run() {
    let mut lines = BufReader::new(stdin()).lines();

    assert_eq!(lines.next_line().await.unwrap().unwrap(), "tei");
    println!("id name cataklysm");
    println!("id author alion02");
    println!("teiok");

    let send = unbounded_channel::<Search>();
    let recv = unbounded_channel::<Box<dyn Game>>();
    spawn(move || {
        let mut rx = send.1;
        let tx = recv.0;

        while let Some(Search {
            mut game,
            time_target,
        }) = rx.blocking_recv()
        {
            let start = Instant::now();
            let mut action;
            let mut depth_times = [0.0f64; 3];
            let mut d = 1;

            loop {
                let eval;
                (eval, action) = game.search(d);

                let elapsed = start.elapsed();

                // FIXME: Mate scores
                println!("info depth {} pv {} score cp {}", d, action, eval.raw(),);

                if game.clear_abort_flag() {
                    println!("info string search aborted");
                    break;
                }

                if eval.is_decisive() {
                    break;
                }

                depth_times.rotate_left(1);
                depth_times[2] = elapsed.as_secs_f64();
                d += 1;

                let expected_time = depth_times[2] * depth_times[1] / depth_times[0];
                let expected_time = if expected_time.is_finite() {
                    Duration::from_secs_f64(expected_time)
                } else {
                    Duration::ZERO
                };

                if expected_time > time_target {
                    break;
                }
            }

            println!("bestmove {action}");
            tx.send(game).unwrap();
        }
    });

    let mut state = State {
        rx: recv.1,
        tx: send.0,
        history: Vec::with_capacity(256),
        game: None,
        flag: None,
    };

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

                        state.abort().await;
                        state.history.clear();
                        state.game = Some(new_game(size, Options::default(size).unwrap()).unwrap());
                    }
                    "position" => {
                        assert_eq!(cmd.next().unwrap(), "startpos");
                        assert_eq!(cmd.next().unwrap(), "moves");

                        assert!(
                            state.history
                                .iter()
                                .zip(cmd.by_ref())
                                .all(|(curr, new)| curr == new),
                            "undo not yet supported",
                        );

                        let game = state.game.as_mut().unwrap();
                        for m in cmd {
                            let action = game.parse_action(m).unwrap();
                            game.play(action).unwrap();
                            state.history.push(m.to_string());
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

                        // TODO: Compute time target
                        let time_target = Duration::from_secs(5);

                        state.abort().await;

                        let Some(mut game) = state.game.take() else {
                            panic!("no game to search on")
                        };
                        state.flag = Some(game.abort_flag());
                        state.tx.send(Search { game, time_target }).unwrap();
                    }
                    "quit" => {
                        state.abort().await;
                        break;
                    }
                    "stop" => state.abort().await,
                    _ => panic!(r#"unsupported command "{line}""#),
                }
            }
            game = state.rx.recv() => {
                state.flag = None;
                state.game = Some(game.unwrap());
            }
        }
    }
}

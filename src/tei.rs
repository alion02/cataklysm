use std::{pin::Pin, thread::spawn};

use cataklysm::{game::*, pair::Pair};

use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{sleep, Duration, Instant, Sleep},
};

// FIXME
const FOREVER: Duration = Duration::from_secs(60 * 60 * 24 * 365); // 1 year
const ABORT_MARGIN: Duration = Duration::from_millis(20);

struct State {
    rx: UnboundedReceiver<Box<dyn Game>>,
    tx: UnboundedSender<Search>,
    history: Vec<String>, // TODO: Use Box<dyn Action>?
    game: Option<Box<dyn Game>>,
    flag: Option<AbortFlag>,
    debug: bool,
    timeout: Pin<Box<Sleep>>,
}

impl State {
    async fn abort(&mut self) {
        if let Some(flag) = self.flag.take() {
            flag.set();

            let start = Instant::now();

            self.game = Some(self.rx.recv().await.unwrap());
            self.timeout.as_mut().reset(start + FOREVER);

            if self.debug {
                println!("info string search aborted in {:?}", start.elapsed());
            }
        }
    }
}

struct Search {
    game: Box<dyn Game>,
    start: Instant,
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
            start,
            time_target,
        }) = rx.blocking_recv()
        {
            let mut action;
            let mut depth_times = [0.0f64; 3];
            let mut d = 1;

            loop {
                let eval;
                (eval, action) = game.search(d);

                let elapsed = start.elapsed();

                // FIXME: Mate scores
                println!(
                    "info depth {} time {} pv {} score cp {}",
                    d,
                    elapsed.as_millis(),
                    action,
                    eval.raw(),
                );

                if game.clear_abort_flag() {
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
        debug: false,
        timeout: Box::pin(sleep(FOREVER)),
    };

    loop {
        select! {
            biased;

            game = state.rx.recv() => {
                state.flag = None;
                state.game = Some(game.unwrap());
            }
            _ = state.timeout.as_mut() => state.abort().await,
            line = lines.next_line() => {
                let line = line.unwrap().unwrap();
                let mut cmd = line.split_ascii_whitespace();
                match cmd.next().unwrap() {
                    "isready" => println!("readyok"),
                    "debug" => {
                        state.debug = match cmd.next().unwrap() {
                            "on" => true,
                            "off" => false,
                            _ => panic!("malformed debug command"),
                        };
                    }
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

                        state.abort().await;
                        let game = state.game.as_mut().expect("can't switch position");
                        for m in cmd {
                            let action = game.parse_action(m).unwrap();
                            game.play(action).unwrap();
                            state.history.push(m.to_string());
                        }
                    }
                    "go" => {
                        let start = Instant::now();

                        state.abort().await;
                        let mut game = state.game.take().expect("can't start search");

                        let mut time = Pair::default();
                        let mut increment = Pair::default();

                        while let Some(subcmd) = cmd.next() {
                            let target = match subcmd {
                                "wtime" => &mut time.white,
                                "btime" => &mut time.black,
                                "winc" => &mut increment.white,
                                "binc" => &mut increment.black,
                                _ => panic!(r#"unsupported command "{line}" @ "{subcmd}""#),
                            };

                            *target = Duration::from_millis(cmd.next().unwrap().parse().unwrap());
                        }

                        let color = game.active_color();

                        // TODO: Improve?
                        // Assume 2/3 of the moves are placements.
                        let expected_moves_left = game.stones_left()[color] * 3 / 2;

                        // FIXME: Does not handle 0 increment well. Negative Duration not allowed.
                        let time_target = (time[color] + increment[color] * expected_moves_left) /
                            expected_moves_left;

                        state.flag = Some(game.abort_flag());
                        game.clear_abort_flag();
                        state.tx.send(Search { game, start, time_target }).unwrap();

                        state.timeout.as_mut().reset(
                            start + time[color] - ABORT_MARGIN
                        );

                        if state.debug {
                            println!("info string target time = {time_target:?}");
                        }
                    }
                    "quit" => {
                        state.abort().await;
                        break;
                    }
                    "stop" => state.abort().await,
                    _ => panic!(r#"unsupported command "{line}""#),
                }
            }
        }
    }
}

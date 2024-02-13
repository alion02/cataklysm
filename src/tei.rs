use std::{pin::Pin, thread::spawn};

use crate::game::new_game;
use common::{game::*, pair::Pair};

use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{sleep, Duration, Instant, Sleep},
};

// FIXME
const FOREVER: Duration = Duration::from_secs(60 * 60 * 24 * 365); // 1 year
const ABORT_MARGIN: Duration = Duration::from_millis(20);
const MAX_DEPTH: u32 = 60;
const IGNORE_ABORT_DEPTH: u32 = 2;
const MIN_KOMI: i32 = -20;
const MAX_KOMI: i32 = 20;

struct State {
    rx: UnboundedReceiver<Box<dyn Game>>,
    tx: UnboundedSender<Search>,
    history: Vec<String>, // TODO: Use Box<dyn Action>?
    game: Option<Box<dyn Game>>,
    flag: Option<AbortFlag>,
    debug: bool,
    timeout: Pin<Box<Sleep>>,
    half_komi: i32,
}

impl State {
    fn new(rx: UnboundedReceiver<Box<dyn Game>>, tx: UnboundedSender<Search>) -> Self {
        Self {
            rx,
            tx,
            history: Vec::with_capacity(256),
            game: None,
            flag: None,
            debug: false,
            timeout: Box::pin(sleep(FOREVER)),
            half_komi: 0,
        }
    }

    async fn abort(&mut self) {
        let start = Instant::now();

        if let Some(flag) = self.flag.take() {
            flag.set();

            self.game = Some(self.rx.recv().await.unwrap());

            if self.debug {
                println!("info string search aborted in {:?}", start.elapsed());
            }
        }

        self.timeout.as_mut().reset(start + FOREVER);
    }

    async fn handle_command(&mut self, line: &str) -> bool {
        let mut cmd = line.split_ascii_whitespace();
        match cmd.next().unwrap() {
            "isready" => println!("readyok"),
            "debug" => {
                self.debug = match cmd.next().unwrap() {
                    "on" => true,
                    "off" => false,
                    _ => panic!("malformed debug command"),
                };
            }
            "setoption" => {
                assert_eq!(cmd.next().unwrap(), "name");
                assert_eq!(cmd.next().unwrap(), "HalfKomi");
                assert_eq!(cmd.next().unwrap(), "value");

                self.half_komi = cmd.next().unwrap().parse().unwrap();
                assert!(self.half_komi >= MIN_KOMI);
                assert!(self.half_komi <= MAX_KOMI);
            }
            "teinewgame" => {
                let size = cmd.next().unwrap().parse().unwrap();

                self.abort().await;
                self.history.clear();
                self.game = Some(
                    new_game(
                        size,
                        Options {
                            half_komi: self.half_komi,
                            ..Options::default(size).unwrap()
                        },
                    )
                    .unwrap(),
                );
            }
            "position" => {
                assert_eq!(cmd.next().unwrap(), "startpos");
                assert_eq!(cmd.next().unwrap(), "moves");

                assert!(
                    self.history
                        .iter()
                        .zip(cmd.by_ref())
                        .all(|(curr, new)| curr == new),
                    "undo not yet supported",
                );

                self.abort().await;
                let game = self.game.as_mut().expect("can't switch position");
                for mv in cmd {
                    let action = game.parser()(mv).unwrap();
                    game.play(action).unwrap();
                    self.history.push(mv.to_string());
                }
            }
            "go" => {
                let start = Instant::now();

                self.abort().await;
                let mut game = self.game.take().expect("can't start search");

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
                let time_target = (time[color] + increment[color] * expected_moves_left)
                    / (expected_moves_left + 1);

                self.flag = Some(game.abort_flag());
                game.clear_abort_flag();
                self.tx
                    .send(Search {
                        game,
                        start,
                        time_target,
                    })
                    .unwrap();

                self.timeout
                    .as_mut()
                    .reset(start + time[color] - ABORT_MARGIN);

                if self.debug {
                    println!("info string target time = {time_target:?}");
                }
            }
            "quit" => {
                self.abort().await;
                return true;
            }
            "stop" => self.abort().await,
            _ => panic!(r#"unsupported command "{line}""#),
        }

        false
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
    println!("option name HalfKomi type spin default 0 min {MIN_KOMI} max {MAX_KOMI}");
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

            // Prevent reading the abort flag at the start
            game.swap_abort_flags();

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

                // Restore the abort flag if we reach the target minimum depth
                if d == IGNORE_ABORT_DEPTH {
                    game.swap_abort_flags();
                }

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

                if d > MAX_DEPTH {
                    break;
                }
            }

            println!("bestmove {action}");
            tx.send(game).unwrap();
        }
    });

    let mut state = State::new(recv.1, send.0);

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
                if state.handle_command(&line).await {
                    break;
                }
            }
        }
    }
}

use std::{
    sync::atomic::Ordering::Relaxed,
    thread::spawn,
    time::{Duration, Instant},
};

use cataklysm::game::*;

use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select,
    sync::mpsc::{error::TryRecvError, unbounded_channel},
};

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

                if game
                    .abort_flag()
                    .compare_exchange(true, false, Relaxed, Relaxed)
                    .is_ok()
                {
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

    // let rx = recv.1;
    // let tx = send.0;

    // let mut history = vec![];

    // loop {
    //     select! {
    //         biased;

    //         line = lines.next_line() => {
    //             let line = line.unwrap().unwrap();
    //             let mut cmd = line.split_ascii_whitespace();
    //             match cmd.next().unwrap() {
    //                 "isready" => println!("readyok"),
    //                 "teinewgame" => {
    //                     let size = cmd.next().unwrap().parse().unwrap();
    //                     let game = new_game(size, Options::default(size).unwrap()).unwrap();
    //                     tx.send(Game(game)).unwrap();
    //                 }
    //                 "position" => {
    //                     assert_eq!(cmd.next().unwrap(), "startpos");
    //                     assert_eq!(cmd.next().unwrap(), "moves");

    //                     assert!(
    //                         history.iter().zip(cmd.by_ref()).all(|(curr, new)| curr == new),
    //                         "undo not yet supported",
    //                     );

    //                     for m in cmd {
    //                         // TODO: Sincere apologies
    //                         tx.send(Play(m.to_string())).unwrap();
    //                         history.push(m.to_string());
    //                     }
    //                 }
    //                 "go" => {
    //                     let time = Pair::default();
    //                     let increment = Pair::default();

    //                     while let Some(subcmd) = cmd.next() {
    //                         let target = &mut match subcmd {
    //                             "wtime" => time.white,
    //                             "btime" => time.black,
    //                             "winc" => increment.white,
    //                             "binc" => increment.black,
    //                             _ => panic!(r#"unsupported command "{line}" @ "{subcmd}""#),
    //                         };

    //                         *target = Duration::from_millis(cmd.next().unwrap().parse().unwrap());
    //                     }

    //                     tx.send(Search { time, increment }).unwrap();
    //                 }

    //                 _ => panic!(r#"unsupported command "{line}""#),
    //             }
    //         }
    //     }
    // }
}

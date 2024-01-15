use std::{
    env::{args, Args},
    time::Instant,
};

use cataklysm::game::*;

fn main() {
    let mut args = args();

    let make_game = |mut args: Args| {
        let time = Instant::now();
        let tps = args.next().unwrap();
        let game = new_game(Options::from_position(Position::Tps(&tps)).unwrap()).unwrap();
        println!(
            "initialized in {:.1}ms",
            time.elapsed().as_secs_f64() * 1000.,
        );
        game
    };

    match args.nth(1).unwrap().as_str() {
        "perft" => {
            let mut game = make_game(args);
            for d in 1.. {
                for mode in [PerftMode::Batch, PerftMode::Naive] {
                    let time = Instant::now();
                    let nodes = game.perft(d, mode);
                    let secs = time.elapsed().as_secs_f64();
                    let nps = nodes as f64 / secs;

                    println!(
                        "depth {d}: {nodes} nodes in {secs:.2}s ({:.1} Mnps, {})",
                        nps / 1_000_000.,
                        match mode {
                            PerftMode::Naive => "naive",
                            PerftMode::Batch => "batch",
                        },
                    );
                }
            }
        }
        "search" => {
            let mut game = make_game(args);
            for d in 1..30 {
                let time = Instant::now();
                let (eval, action) = game.search(d);
                let secs = time.elapsed().as_secs_f64();

                println!(
                    "depth {d}: {} (eval: {eval}) in {secs:.2}s",
                    action.unwrap(),
                );
            }
        }
        _ => panic!(),
    }
}

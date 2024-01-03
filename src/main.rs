use std::{env::args, time::Instant};

use takkit::game::*;

fn main() {
    let mut args = args();

    match args.nth(1).unwrap().as_str() {
        "perft" => {
            let tps = args.next().unwrap();
            let mut game = new_game(Options::from_position(Position::Tps(&tps)).unwrap()).unwrap();
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
        _ => panic!(),
    }
}
use std::{
    env::{args, Args},
    io::stdin,
    time::Instant,
};

use cataklysm::game::*;

fn main() {
    let mut args = args();

    let Some(mode) = args.nth(1) else {
        return help();
    };

    match mode.as_str() {
        "perft" => perft(args),
        "search" => search(args),
        "showmatch" => showmatch(args),
        "hashtest" => hashtest(args),
        _ => help(),
    }
}

fn perft(args: Args) {
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

fn search(args: Args) {
    let mut game = make_game(args);
    for d in 1..30 {
        let time = Instant::now();
        let (eval, action) = game.search(d);
        let secs = time.elapsed().as_secs_f64();
        let nodes = game.take_nodes();
        let nps = nodes as f64 / secs;

        println!(
            "depth {d}: {} (eval: {eval}), {nodes} nodes in {secs:.2}s ({:.2} Mnps)",
            action.unwrap(),
            nps / 1_000_000.,
        );

        if eval.is_decisive() {
            break;
        }
    }
}

fn showmatch(args: Args) {
    let mut game = make_game(args);
    loop {
        let mut chosen = None;
        for d in 1..8 {
            let (eval, action) = game.search(d);
            chosen = action;
            if eval.is_decisive() {
                break;
            }
        }

        let Some(chosen) = chosen else {
            println!("game finished");
            break;
        };

        println!("{chosen}");
        game.play(chosen).unwrap();
    }
}

fn hashtest(args: Args) {
    let mut game = make_game(args);
    let mut stdin = stdin().lines().map(|l| l.unwrap());
    loop {
        println!("{:?}", game.curr_hash());

        let Ok(action) = game.parse_action(&stdin.next().unwrap()) else {
            println!("could not parse action");
            continue;
        };

        if game.play(action).is_err() {
            println!("illegal action");
        }
    }
}

fn help() {
    println!(
        r#"usage:
    perft "<tps>"
    search "<tps>"
    showmatch "<tps>"
    hashtest "<tps>"
    "#
    );
}

fn make_game(mut args: Args) -> Box<dyn Game> {
    let time = Instant::now();
    let tps = args.next().unwrap();
    let game = new_game(Options::from_position(Position::Tps(&tps)).unwrap()).unwrap();
    println!(
        "initialized in {:.1}ms",
        time.elapsed().as_secs_f64() * 1000.,
    );
    game
}

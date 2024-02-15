mod game;
mod tei;

use std::{
    env::{args, Args},
    io::stdin,
    time::Instant,
};

use crate::game::new_game;
use common::game::*;

use tokio::runtime::Builder;

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
        "tei" => Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(tei::run()),
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
        game.clear_nodes();
        let (eval, action) = game.search(d);
        let secs = time.elapsed().as_secs_f64();
        let nodes = game.nodes();
        let nps = nodes as f64 / secs;

        println!(
            "depth {d}: {} (eval: {eval}), {nodes} nodes in {secs:.2}s ({:.2} Mnps)",
            action,
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
        let mut action;
        let mut d = 1;
        loop {
            let eval;
            (eval, action) = game.search(d);

            if eval.is_decisive() {
                break;
            }

            d += 1;
            if d > 7 {
                break;
            }
        }

        println!("{action}");
        game.play(action).unwrap();

        // FIXME: Aborts game too early
        if d == 1 {
            println!("game finished");
            break;
        }
    }
}

fn hashtest(args: Args) {
    let mut game = make_game(args);
    let mut stdin = stdin().lines().map(|l| l.unwrap());
    loop {
        println!("{:?}", game.hash());

        let Ok(action) = game.parser()(&stdin.next().unwrap()) else {
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
    tei
    perft "<tps>"
    search "<tps>"
    showmatch "<tps>"
    hashtest "<tps>""#
    );
}

fn make_game(mut args: Args) -> Box<dyn Game> {
    let time = Instant::now();
    let tps = &args.next().unwrap();
    let size = size_of_tps(tps);
    let mut game = new_game(size, Options::default(size).unwrap()).unwrap();
    game.set_position(tps).unwrap();
    println!(
        "initialized in {:.1}ms",
        time.elapsed().as_secs_f64() * 1000.,
    );
    game
}

#[cfg(all(
    target_arch = "x86_64",
    not(target_feature = "avx2"),
    not(feature = "allow-old-x86-64"),
))]
mod slow_error {
    #[cfg(windows)]
    compile_error!(
        r#"cataklysm compiled for x86-64 without support for modern instructions is slow

if this is not desired, set the RUSTFLAGS environment variable appropriately and re-run the build:
    in command prompt: set RUSTFLAGS=-Ctarget-cpu=native
    in powershell: $env:RUSTFLAGS=-Ctarget-cpu=native

otherwise, or if the error persists, enable the feature "allow-old-x86-64" for the build, for example:
    cargo b -r --features allow-old-x86-64

"#
    );

    #[cfg(not(windows))]
    compile_error!(
        r#"cataklysm compiled for x86-64 without support for modern instructions is slow

if this is not desired, re-run the build with the appropriate RUSTFLAGS environment variable, for example:
    RUSTFLAGS=-Ctarget-cpu=native cargo b -r

otherwise, or if the error persists, enable the feature "allow-old-x86-64" for the build, for example:
    cargo b -r --features allow-old-x86-64

"#
    );
}

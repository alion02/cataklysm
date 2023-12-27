use std::{
    env::var,
    fs::{read_to_string, write},
    path::Path,
};

fn main() {
    println!("cargo:rerun-if-changed=src/template.rs");

    let out_dir = var("OUT_DIR").unwrap();
    let out_file = Path::new(&out_dir).join("macro.rs");

    let input = read_to_string("src/template.rs").unwrap();
    let (template, _dummy_params) = input.split_once("// $end_template").unwrap();

    let mut result = String::new();

    result += "macro_rules! state {
    ($mod:ident => $export:ident {
        size: $SIZE:expr,
        row_len: $ROW_LEN:expr,
        bitboard: $Bitboard:ident,
        stack: $Stack:ident,
        action: $ActionBacking:ident,
        perft: $PERFT:expr,
    }) => {
        pub(crate) use $mod::State as $export;
        mod $mod {";

    result += &template;

    result += "
const SIZE: usize = $SIZE;
const ROW_LEN: usize = $ROW_LEN;

type Bitboard = $Bitboard;
type Stack = $Stack;
type ActionBacking = $ActionBacking;

const PERFT: &[(u32, u64)] = &$PERFT;";

    result += "
        }
    };
}";

    write(out_file, result).unwrap();
}

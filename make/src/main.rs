use std::{
    fmt::Write,
    fs::{create_dir_all, remove_dir_all, write},
    path::PathBuf,
};

use anyhow::*;

fn main() -> Result<()> {
    _ = remove_dir_all("./crates");

    let crates: Vec<_> = ('3'..'9')
        .map(|s| format!("size{s}").into_boxed_str())
        .collect();

    let mut workspace = String::new();

    for c in crates {
        let size = PathBuf::from(format!("./crates/{c}/Cargo.toml"));
        create_dir_all(size.parent().unwrap())?;
        write(
            size,
            format!(
                r#"[package]
name = "{c}"
version = "0.1.0"
edition = "2021"

[lib]
path = "../../generic/lib.rs"
"#,
            ),
        )?;

        writeln!(workspace, r#"    "crates/{c}","#)?;
    }

    write(
        "Cargo.toml",
        format!(
            r#"[workspace]
resolver = "2"
members = [
    "make",
{workspace}]
"#,
        ),
    )?;

    Ok(())
}

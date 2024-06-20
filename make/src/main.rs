use std::{
    fmt::Write,
    fs::{create_dir_all, remove_dir_all, write},
    path::PathBuf,
};

use anyhow::*;

fn main() -> Result<()> {
    _ = remove_dir_all("./crates");

    let mut workspace = String::new();

    for s in '3'..'9' {
        let c = format!("size{s}");

        let crate_path = PathBuf::from(format!("./crates/{c}/Cargo.toml"));
        create_dir_all(crate_path.parent().unwrap())?;
        write(
            crate_path,
            format!(
                r#"[package]
name = "{c}"
version = "0.1.0"
edition = "2021"

[lib]
path = "../../generic/lib.rs"

[features]
default = ["{s}"]
{s} = []
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

#![allow(
    clippy::format_collect, // Performance is irrelevant
)]

use std::fs::{create_dir_all, remove_dir_all, write};

use anyhow::*;

fn main() -> Result<()> {
    _ = remove_dir_all("crates");

    let crates = (3..9).map(|size| (size, format!("size{size}"), format!("crates/size{size}")));

    for (size, name, path) in crates.clone() {
        create_dir_all(&path)?;
        write(
            format!("{path}/Cargo.toml"),
            format!(
                r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
path = "../../generic/lib.rs"

[features]
default = ["{size}"]
{size} = []
"#,
            ),
        )?;
    }

    write(
        "Cargo.toml",
        format!(
            r#"[workspace]
resolver = "2"
members = [
    "make",
{}]
"#,
            crates
                .map(|(_size, _name, path)| format!("    \"{path}\",\n"))
                .collect::<String>()
        ),
    )?;

    Ok(())
}

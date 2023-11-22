use std::{
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process,
};

use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Deserialize;

mod package;

static WORKSPACE_CONFIG: Lazy<Cargo> = Lazy::new(|| {
    match (|| -> Result<Cargo> {
        let mut file = File::open(project_root().join("Cargo.toml"))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(toml::from_str::<Cargo>(&buf)?)
    })() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            process::exit(-1);
        }
    }
});

#[derive(Deserialize)]
struct Cargo {
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
    repository: String,
}

fn main() -> Result<()> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("package") => package::main()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Usage: cargo xtask <TASK>

TASKS:
    package         build application, man page and shell completion scripts"
    )
}

pub fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

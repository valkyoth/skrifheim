#![forbid(unsafe_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("check") => run_script("scripts/checks.sh"),
        Some("help") | None => {
            eprintln!("usage: cargo run -p xtask -- check");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("unknown xtask command: {other}");
            ExitCode::from(2)
        }
    }
}

fn run_script(path: &str) -> ExitCode {
    let script = workspace_root().join(path);
    match Command::new(&script).status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(status) => match status.code() {
            Some(0) => ExitCode::SUCCESS,
            Some(_) => ExitCode::from(1),
            None => ExitCode::from(1),
        },
        Err(error) => {
            eprintln!("failed to run {}: {error}", script.display());
            ExitCode::from(1)
        }
    }
}

fn workspace_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir)
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

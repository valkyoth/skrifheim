#![forbid(unsafe_code)]

use std::env;
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
    match Command::new(path).status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(status) => match status.code() {
            Some(code) => ExitCode::from(code as u8),
            None => ExitCode::from(1),
        },
        Err(error) => {
            eprintln!("failed to run {path}: {error}");
            ExitCode::from(1)
        }
    }
}

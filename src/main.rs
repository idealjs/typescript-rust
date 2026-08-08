//! Entry point for the `tsox` binary.
//!
//! Mirrors `cmd/tsgo/main.go` in the Go implementation: dispatches `--lsp` and
//! `--api` subcommands (stubbed for now), otherwise runs the `tsc` compilation
//! pipeline via `execute::command_line`.

use std::process::ExitCode;
use std::thread;

use tsox::execute::{OsSystem, command_line};

fn main() -> ExitCode {
    // Run on a thread with a large stack (256 MB) to handle deep recursion
    // in the type checker. Go goroutines grow their stack dynamically up to
    // 1 GB; Rust's default 8 MB is insufficient for deeply recursive type
    // comparisons. The algorithmic depth limits (RELATER_MAX_DEPTH=100,
    // MAX_SERIALIZATION_LEVEL=2) should prevent overflow in normal cases,
    // but this provides a safety margin for edge cases.
    let result = thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(main_inner)
        .unwrap()
        .join()
        .unwrap();
    result
}

fn main_inner() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Dispatch special subcommands.
    if let Some(first) = args.first() {
        match first.as_str() {
            "--lsp" => {
                let code = tsox::lsp::run_lsp();
                return ExitCode::from(code as u8);
            }
            "--api" => {
                let code = tsox::api::run_api();
                return ExitCode::from(code as u8);
            }
            _ => {}
        }
    }

    let sys = OsSystem::new();
    let result = command_line(&sys, &args);
    ExitCode::from(result.status.as_i32() as u8)
}

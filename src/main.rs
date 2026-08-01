//! Entry point for the `tsox` binary.
//!
//! Mirrors `cmd/tsgo/main.go` in the Go implementation: dispatches `--lsp` and
//! `--api` subcommands (stubbed for now), otherwise runs the `tsc` compilation
//! pipeline via `execute::command_line`.

use std::process::ExitCode;

use tsox::execute::{OsSystem, command_line};

fn main() -> ExitCode {
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

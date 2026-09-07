use std::process::ExitCode;
use std::thread;

use tsox::execute::{OsSystem, command_line};

fn main() -> ExitCode {
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

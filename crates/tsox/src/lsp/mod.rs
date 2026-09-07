pub mod dynamic_queue;
pub mod logger;
pub mod lsproto;
pub mod lspwatcher;
pub mod progress;
pub mod server;
pub mod stack_sanitizer;

mod document_symbols;
mod features;
mod handlers;
mod lsp_server;
mod refs;
mod symbol_nav;
mod utils;

pub use lsp_server::{LspServer, run_lsp};

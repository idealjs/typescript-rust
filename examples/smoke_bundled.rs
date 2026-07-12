//! Smoke test: parse all bundled lib files and report diagnostic counts.
use tsox::bundled::{lib_contents, lib_names};
use tsox::parser::Parser;

fn main() {
    let names = lib_names();
    println!("Total bundled libs: {}", names.len());
    let mut total_errors = 0;
    let mut failed = Vec::new();
    for name in &names {
        let contents = match lib_contents(name) {
            Some(c) => c,
            None => continue,
        };
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics(name, contents.to_string());
        if !diagnostics.is_empty() {
            total_errors += diagnostics.len();
            failed.push((*name, diagnostics.len()));
            println!("FAIL {name}: {} diagnostics", diagnostics.len());
            for d in diagnostics.iter().take(5) {
                println!(
                    "  code={} pos={}-{}: {:?}",
                    d.message.code,
                    d.range.pos(),
                    d.range.end(),
                    d.message_args
                );
            }
        }
    }
    println!("\n=== Summary ===");
    println!("Total libs: {}", names.len());
    println!("Failed libs: {}", failed.len());
    println!("Total diagnostics: {}", total_errors);
    if total_errors == 0 {
        println!("ALL BUNDLED LIBS PARSE WITH 0 DIAGNOSTICS");
    }
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.ReplaceLine"]
#[test]
fn incremental_parsing_top_level_await1() {
    let content = r#"// @target: esnext
// @module: esnext
// @Filename: ./foo.ts
await(1);
/*1*/"#;
    let mut s = Session::new(content);
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "export {};");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 0);
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 1, "")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}

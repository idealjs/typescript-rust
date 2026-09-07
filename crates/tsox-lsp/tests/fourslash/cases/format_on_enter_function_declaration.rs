use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn format_on_enter_function_declaration() {
    let content = r#"/*0*/function listAPIFiles(path: string): string[] {/*1*/ }"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::go_to_marker(&mut s, "0");
    fourslash::verify_current_line_content(
        &mut s,
        r#"function listAPIFiles(path: string): string[] {"#,
    );
    // TODO: }
}

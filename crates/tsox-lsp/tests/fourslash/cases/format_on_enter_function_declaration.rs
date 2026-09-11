use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_on_enter_function_declaration() {
    let content = r#"/*0*/function listAPIFiles(path: string): string[] {/*1*/ }"#;
    let mut s = Session::new_for_test("formatOnEnterFunctionDeclaration", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.InsertLine(t, "")
    fourslash::go_to_marker(&mut s, "0");
    fourslash::verify_current_line_content(&mut s, r#"function listAPIFiles(path: string): string[] {"#);
    // TODO: }
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn format_bracket_in_switch_case() {
    let content = r#"// @lib: es5
switch (x) {
    case[]:
}"#;
    let mut s = Session::new_for_test("formatBracketInSwitchCase", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"switch (x) {
    case []:
}"#);
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn as_operator_formatting() {
    let content = r#"/**/var x = 3   as  number;"#;
    let mut s = Session::new_for_test("asOperatorFormatting", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_line_content(&mut s, r#"var x = 3 as number;"#);
}

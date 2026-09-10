use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_error_types1() {
    let content = r#"var /*A*/f: {
    x: number;
    <
};"#;
    let mut s = Session::new_for_test("quickInfoOnErrorTypes1", content);
    fourslash::verify_quick_info_at(&mut s, "A", "var f: {\n    (): any;\n    x: number;\n}", "");
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_syntax_error_no_error() {
    let content = r#"namespace X {
    export =
}
X.add/*1*/"#;
    let mut s = Session::new_for_test("quickInfoForSyntaxErrorNoError", content);
    fourslash::verify_quick_info_at(&mut s, "1", "any", "");
}

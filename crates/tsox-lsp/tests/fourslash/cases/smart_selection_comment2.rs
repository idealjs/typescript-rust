use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_comment2() {
    let content = r#"const a = 1; //a b/**/c d"#;
    let mut s = Session::new_for_test("smartSelection_comment2", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}

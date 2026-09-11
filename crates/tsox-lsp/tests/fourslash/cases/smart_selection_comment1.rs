use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_comment1() {
    let content = r#"const a = 1; ///**/comment content"#;
    let mut s = Session::new_for_test("smartSelection_comment1", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}

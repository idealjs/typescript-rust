use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_comment1() {
    let content = r#"const a = 1; ///**/comment content"#;
    let _s = Session::new_for_test("smartSelection_comment1", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}

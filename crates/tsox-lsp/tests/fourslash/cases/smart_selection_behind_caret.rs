use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_behind_caret() {
    let content = r#"let/**/ x: string"#;
    let _s = Session::new_for_test("smartSelection_behindCaret", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}

use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_last_blank_line() {
    let content = r#"class C {}
/**/"#;
    let _s = Session::new_for_test("smartSelection_lastBlankLine", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}

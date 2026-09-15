use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_template_strings2() {
    let content = r#"`a ${b} /**/c`"#;
    let _s = Session::new_for_test("smartSelection_templateStrings2", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}

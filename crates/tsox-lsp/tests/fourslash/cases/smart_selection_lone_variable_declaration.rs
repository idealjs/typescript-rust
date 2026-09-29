use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_lone_variable_declaration() {
    let content = r#"const /**/x = 3;"#;
    let _s = Session::new_for_test("smartSelection_loneVariableDeclaration", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}

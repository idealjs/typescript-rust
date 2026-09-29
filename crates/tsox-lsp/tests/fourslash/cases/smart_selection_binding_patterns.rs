use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_binding_patterns() {
    let content = r#"const { /*1*/x, y: /*2*/a, .../*3*/zs = {} } = {};"#;
    let _s = Session::new_for_test("smartSelection_bindingPatterns", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}

use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_function3() {
    let content = r#"const f3 = function () {
    /**/
}"#;
    let _s = Session::new_for_test("smartSelection_function3", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}

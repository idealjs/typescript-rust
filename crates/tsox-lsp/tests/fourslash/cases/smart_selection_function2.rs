use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_function2() {
    let content = r#"function f2() {
    /**/
}"#;
    let _s = Session::new_for_test("smartSelection_function2", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}

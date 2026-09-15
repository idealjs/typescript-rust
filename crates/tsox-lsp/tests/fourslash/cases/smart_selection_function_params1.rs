use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_function_params1() {
    let content = r#"function f(/*1*/p, /*2*/q?, /*3*/...r: any[] = []) {}"#;
    let _s = Session::new_for_test("smartSelection_functionParams1", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}

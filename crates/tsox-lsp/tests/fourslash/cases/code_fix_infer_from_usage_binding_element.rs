use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_infer_from_usage_binding_element() {
    let content = r#"function f([car, cdr]) {
    return car + cdr + 1
}"#;
    let _s = Session::new_for_test("codeFixInferFromUsageBindingElement", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}

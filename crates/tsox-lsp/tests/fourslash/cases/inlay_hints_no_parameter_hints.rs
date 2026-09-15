use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_no_parameter_hints() {
    let content = r#"function foo (a: number, b: number) {}
foo(1, 2);"#;
    let _s = Session::new_for_test("inlayHintsNoParameterHints", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

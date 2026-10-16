use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_type_parameter_modifiers1() {
    let content = r#"function test1() {
  return function <const T>(a: T) {};
}"#;
    let _s = Session::new_for_test("inlayHintsTypeParameterModifiers1", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

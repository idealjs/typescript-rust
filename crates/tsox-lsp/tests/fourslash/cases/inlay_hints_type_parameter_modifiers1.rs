use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_type_parameter_modifiers1() {
    let content = r#"function test1() {
  return function <const T>(a: T) {};
}"#;
    let mut s = Session::new_for_test("inlayHintsTypeParameterModifiers1", content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

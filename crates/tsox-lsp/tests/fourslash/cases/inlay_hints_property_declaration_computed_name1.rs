use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_property_declaration_computed_name1() {
    let content = r#"function foo() {
  const sym = Symbol();
  class C {
    [sym] = 123;
  }
}"#;
    let mut s = Session::new_for_test("inlayHintsPropertyDeclarationComputedName1", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{
}

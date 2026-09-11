use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_interactive_inferred_type_predicate1() {
    let content = r#"// @strict: true
function test(x: unknown) {
  return typeof x === 'number';
}"#;
    let mut s = Session::new_for_test("inlayHintsInteractiveInferredTypePredicate1", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_inferred_type_predicate1() {
    let content = r#"// @strict: true
function test(x: unknown) {
  return typeof x === 'number';
}"#;
    let _s = Session::new_for_test("inlayHintsInferredTypePredicate1", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

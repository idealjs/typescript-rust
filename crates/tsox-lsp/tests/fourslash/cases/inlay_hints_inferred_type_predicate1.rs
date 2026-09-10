use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_inferred_type_predicate1() {
    let content = r#"// @strict: true
function test(x: unknown) {
  return typeof x === 'number';
}"#;
    let mut s = Session::new_for_test("inlayHintsInferredTypePredicate1", content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

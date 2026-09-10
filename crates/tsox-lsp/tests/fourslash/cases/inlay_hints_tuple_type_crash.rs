use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_tuple_type_crash() {
    let content = r#"function iterateTuples(tuples: [string][]): void {
  tuples.forEach((l) => {})
}"#;
    let mut s = Session::new_for_test("inlayHintsTupleTypeCrash", content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{
}

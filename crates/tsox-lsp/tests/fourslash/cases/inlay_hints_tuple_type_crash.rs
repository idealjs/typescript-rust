use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_tuple_type_crash() {
    let content = r#"function iterateTuples(tuples: [string][]): void {
  tuples.forEach((l) => {})
}"#;
    let _s = Session::new_for_test("inlayHintsTupleTypeCrash", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{
}

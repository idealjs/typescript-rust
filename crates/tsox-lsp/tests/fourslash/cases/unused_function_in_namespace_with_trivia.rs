use tsox_lsp::fourslash::{self, Session};


#[test]
fn unused_function_in_namespace_with_trivia() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
[| namespace greeter {
  // Do not remove
  /**
   * JSDoc Comment
   */
  function function1() {
  }/*1*/
} |]"#;
    let mut s = Session::new_for_test("unusedFunctionInNamespaceWithTrivia", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace greeter {
}

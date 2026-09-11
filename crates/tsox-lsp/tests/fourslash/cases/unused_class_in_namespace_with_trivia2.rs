use tsox_lsp::fourslash::{self, Session};


#[test]
fn unused_class_in_namespace_with_trivia2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
[| namespace greeter {
  // Do not remove
  /**
   * JSDoc Comment
   */
  class /* comment2 */ class1 {
  }
} |]"#;
    let mut s = Session::new_for_test("unusedClassInNamespaceWithTrivia2", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace greeter {
}

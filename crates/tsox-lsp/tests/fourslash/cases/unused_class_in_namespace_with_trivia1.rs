use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_class_in_namespace_with_trivia1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
[| namespace greeter {
  /* comment1 */
  class /* comment2 */ class1 {
  }
} |]"#;
    let mut s = Session::new_for_test("unusedClassInNamespaceWithTrivia1", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `namespace greeter {
}

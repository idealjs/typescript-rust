use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_class_in_namespace_with_trivia1() {
    let content = r#"// @noUnusedLocals: true
[| namespace greeter {
  /* comment1 */
  class /* comment2 */ class1 {
  }
} |]"#;
    let _s = Session::new_for_test("unusedClassInNamespaceWithTrivia1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace greeter {
}

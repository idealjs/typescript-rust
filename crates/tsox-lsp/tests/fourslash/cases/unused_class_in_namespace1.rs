use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_class_in_namespace1() {
    let content = r#"// @noUnusedLocals: true
[| namespace greeter {
  class class1 {
  }
} |]"#;
    let _s = Session::new_for_test("unusedClassInNamespace1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace greeter {
}

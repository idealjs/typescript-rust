use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_class_in_namespace1() {
    let content = r#"// @noUnusedLocals: true
[| namespace greeter {
  class class1 {
  }
} |]"#;
    let mut s = Session::new_for_test("unusedClassInNamespace1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace greeter {
}

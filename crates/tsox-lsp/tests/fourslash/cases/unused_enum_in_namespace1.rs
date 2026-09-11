use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_enum_in_namespace1() {
    let content = r#"// @noUnusedLocals: true
[| namespace greeter {
  enum enum1 {
      Monday
  }
} |]"#;
    let mut s = Session::new_for_test("unusedEnumInNamespace1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace greeter {
}

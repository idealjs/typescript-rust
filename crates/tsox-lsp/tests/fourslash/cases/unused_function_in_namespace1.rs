use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_function_in_namespace1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
[| namespace greeter {
  // some legit comments
  function function1() {
  }/*1*/
} |]"#;
    let mut s = Session::new_for_test("unusedFunctionInNamespace1", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `namespace greeter {
}

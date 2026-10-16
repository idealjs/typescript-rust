use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_function_in_namespace1() {
    let content = r#"// @noUnusedLocals: true
[| namespace greeter {
  // some legit comments
  function function1() {
  }/*1*/
} |]"#;
    let _s = Session::new_for_test("unusedFunctionInNamespace1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace greeter {
}

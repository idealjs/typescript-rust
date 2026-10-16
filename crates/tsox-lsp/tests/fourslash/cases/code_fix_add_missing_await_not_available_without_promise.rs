use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_add_missing_await_not_available_without_promise() {
    let content = r#"async function fn(a: {}, b: number) {
  a + b;
}"#;
    let _s = Session::new_for_test("codeFixAddMissingAwait_notAvailableWithoutPromise", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "addMissingAwait")
}

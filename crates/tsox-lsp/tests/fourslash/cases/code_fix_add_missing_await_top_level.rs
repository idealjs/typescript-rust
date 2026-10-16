use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_add_missing_await_top_level() {
    let content = r#"declare function getPromise(): Promise<string>;
const p = getPromise();
while (true) {
  p/*0*/.toLowerCase();
  getPromise()/*1*/.toLowerCase();
}"#;
    let _s = Session::new_for_test("codeFixAddMissingAwait_topLevel", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "addMissingAwait")
    // TODO: f.VerifyCodeFixNotAvailable(t, "addMissingAwaitToInitializer")
}

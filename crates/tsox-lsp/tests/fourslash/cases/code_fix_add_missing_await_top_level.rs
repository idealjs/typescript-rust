use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_add_missing_await_top_level() {
    let content = r#"declare function getPromise(): Promise<string>;
const p = getPromise();
while (true) {
  p/*0*/.toLowerCase();
  getPromise()/*1*/.toLowerCase();
}"#;
    let mut s = Session::new_for_test("codeFixAddMissingAwait_topLevel", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "addMissingAwait")
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "addMissingAwaitToInitializer")
}

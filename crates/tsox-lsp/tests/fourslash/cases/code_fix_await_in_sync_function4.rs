use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_await_in_sync_function4() {
    let content = r#"class Foo {
    constructor {
        await Promise.resolve();
    }
}"#;
    let mut s = Session::new_for_test("codeFixAwaitInSyncFunction4", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}

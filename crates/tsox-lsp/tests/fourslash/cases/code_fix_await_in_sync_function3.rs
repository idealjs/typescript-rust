use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_await_in_sync_function3() {
    let content = r#"const f = {
    get a() {
        return await Promise.resolve();
    },
    get a() {
        await Promise.resolve();
    },
}"#;
    let mut s = Session::new_for_test("codeFixAwaitInSyncFunction3", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
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
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}

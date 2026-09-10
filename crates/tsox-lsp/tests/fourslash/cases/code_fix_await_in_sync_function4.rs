use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_await_in_sync_function4() {
    let content = r#"class Foo {
    constructor {
        await Promise.resolve();
    }
}"#;
    let mut s = Session::new_for_test("codeFixAwaitInSyncFunction4", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_await_should_not_crash_if_not_in_function() {
    let content = r#"await a"#;
    let mut s = Session::new_for_test("codeFixAwaitShouldNotCrashIfNotInFunction", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "addMissingAwait")
}

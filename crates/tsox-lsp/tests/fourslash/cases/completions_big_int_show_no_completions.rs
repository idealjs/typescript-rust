use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_big_int_show_no_completions() {
    let content = r#"declare const SSL_OP_SSLEAY_080_CLIENT_DH_BUG: number
const foo = 0n/*1*/;"#;
    let mut s = Session::new_for_test("completionsBigIntShowNoCompletions", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // The assertion here is simply 'does not crash/panic'."]
#[test]
fn completion_jsx_no_crash() {
    let content = r#"
// @filename: file.tsx
<Foo/>/*1*/
"#;
    let mut s = Session::new(content);
    // TODO: // The assertion here is simply "does not crash/panic".
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

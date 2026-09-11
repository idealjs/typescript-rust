use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_jsx_no_crash() {
    let content = r#"
// @filename: file.tsx
<Foo/>/*1*/
"#;
    let mut s = Session::new_for_test("completionJsxNoCrash", content);
    // TODO: // The assertion here is simply "does not crash/panic".
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

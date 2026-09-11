use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_self_declaring3() {
    let content = r#"function f<T extends { x: number }>(p: T & (T extends { hello: string } ? { goodbye: number } : {})) {}
f({ x/*x*/: 0, hello/*hello*/: "", goodbye/*goodbye*/: 0, abc/*abc*/: "" })"#;
    let mut s = Session::new_for_test("completionsSelfDeclaring3", content);
    fourslash::verify_completions_exact_at(&mut s, Some("x"), &["x"]);
    // TODO: f.VerifyCompletions(t, "hello", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("goodbye"), &["goodbye"]);
    // TODO: f.VerifyCompletions(t, "abc", &fourslash.CompletionsExpectedList{
}

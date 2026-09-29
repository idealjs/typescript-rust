use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_literals() {
    let content = r#"const x: 0 | "one" = /**/;
const y: 0 | "one" | 1n = /*1*/;
const y2: 0 | "one" | 1n = 'one'/*2*/;"#;
    let mut s = Session::new_for_test("completionsLiterals", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_for_string_literal_quote_preference4() {
    let content = r#"type T = 0 | 1;
const t: T = /**/"#;
    let mut s = Session::new_for_test("completionForStringLiteral_quotePreference4", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

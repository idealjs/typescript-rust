use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_type_keywords() {
    let content = r#"// @noLib: true
type T = /**/"#;
    let mut s = Session::new_for_test("completionsTypeKeywords", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

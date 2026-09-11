use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_recommended_non_accessible_symbol() {
    let content = r#"function f() {
    class C {}
    return (c: C) => void;
}
f()(new /**/);"#;
    let mut s = Session::new_for_test("completionsRecommended_nonAccessibleSymbol", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

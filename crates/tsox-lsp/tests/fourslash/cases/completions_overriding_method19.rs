use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method19() {
    let content = r#"// @Filename: a.ts
// @newline: LF
class Base {
    method() {}
    protected prop = 1;
}
class E extends Base {
    protected notamodifier override /**/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod19", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}

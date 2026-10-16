use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method21() {
    let content = r#"// @Filename: a.ts
// @newline: LF
abstract class AFoo {
    abstract bar(): Promise<void>;
}
class BFoo extends AFoo {
    async /*b*/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod21", content);
    fourslash::go_to_marker(&mut s, "b");
    // TODO: f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("b"), &fourslash.ApplyCodeActionFromCompletionOptions{
}

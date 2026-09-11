use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method20() {
    let content = r#"// @Filename: a.ts
// @newline: LF
abstract class AFoo {
    abstract bar(): Promise<void>;
}
class Foo extends AFoo {
    async [|b/*a*/|]
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod20", content);
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("a"), &fourslash.ApplyCodeActionFromCompletionOptions{
}

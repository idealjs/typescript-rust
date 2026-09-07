use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("b"), &fourslash.ApplyCodeActionFromCompletionOptions{
}

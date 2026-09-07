use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("a"), &fourslash.ApplyCodeActionFromCompletionOptions{
}

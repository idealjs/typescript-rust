use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_overriding_method7() {
    let content = r#"// @Filename: a.ts
// @newline: LF
abstract class Base {
    abstract M<T>(t: T): void;
    abstract M<T>(t: T, x: number): void;
}

abstract class Derived extends Base {
    abstract /*a*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("a"), &fourslash.ApplyCodeActionFromCompletionOptions{
}

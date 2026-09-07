use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_overriding_method12() {
    let content = r#"// @Filename: a.ts
// @newline: LF
abstract class A {
    public get P(): string {
        return "";
    }
}

abstract class B extends A {
    abstract /*a*/
}

abstract class B1 extends A {
    abstract override /*b*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("b"), &fourslash.ApplyCodeActionFromCompletionOptions{
}

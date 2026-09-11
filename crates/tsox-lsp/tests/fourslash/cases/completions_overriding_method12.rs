use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionsOverridingMethod12", content);
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("b"), &fourslash.ApplyCodeActionFromCompletionOptions{
}

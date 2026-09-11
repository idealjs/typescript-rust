use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method18() {
    let content = r#"// @Filename: a.ts
// @newline: LF
declare function decorator(...args: any[]): any;
class DecoratorBase {
    protected foo(a: string): string;
    protected foo(a: number): number;
    protected foo(a: any): any {
        return a;
    }
}
class DecoratorSub extends DecoratorBase {
    @decorator protected /**/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod18", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}

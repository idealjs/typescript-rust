use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionsOverridingMethod7", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("a"), &fourslash.ApplyCodeActionFromCompletionOptions{
}

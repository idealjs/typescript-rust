use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // Verify completions don't panic on decorator applied to pr"]
#[test]
fn decorator_completion_on_private_method() {
    let content = r#"
// @experimentalDecorators: true
declare function dec(target: any, key: string): void;
class C {
    @dec/**/
    #method() {}
}"#;
    let mut s = Session::new(content);
    // TODO: // Verify completions don't panic on decorator applied to private method
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn decorator_completion_on_private_method() {
    let content = r#"
// @experimentalDecorators: true
declare function dec(target: any, key: string): void;
class C {
    @dec/**/
    #method() {}
}"#;
    let mut s = Session::new_for_test("decoratorCompletionOnPrivateMethod", content);
    // TODO: // Verify completions don't panic on decorator applied to private method
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["dec"], &[]);
}

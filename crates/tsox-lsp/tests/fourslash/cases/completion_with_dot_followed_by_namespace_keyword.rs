use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_with_dot_followed_by_namespace_keyword() {
    let content = r#"namespace A {
    function foo() {
        if (true) {
            B./**/
        namespace B {
            export function baz() { }
}"#;
    let mut s = Session::new_for_test("completionWithDotFollowedByNamespaceKeyword", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: }
}

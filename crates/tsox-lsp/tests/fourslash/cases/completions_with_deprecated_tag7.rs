use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_deprecated_tag7() {
    let content = r#"// @strict: true
interface I {
    /** @deprecated a */
    a: number;
}
const foo = {
    a: 1
}
const i: I = {
    ...foo,
    /**/
}"#;
    let mut s = Session::new_for_test("completionsWithDeprecatedTag7", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

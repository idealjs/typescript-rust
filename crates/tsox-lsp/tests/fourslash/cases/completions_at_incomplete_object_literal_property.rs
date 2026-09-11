use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_at_incomplete_object_literal_property() {
    let content = r#"// @noLib: true
f({
    [|a|]/**/
    xyz: ``,
});
declare function f(options: { abc?: number, xyz?: string }): void;"#;
    let mut s = Session::new_for_test("completionsAtIncompleteObjectLiteralProperty", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

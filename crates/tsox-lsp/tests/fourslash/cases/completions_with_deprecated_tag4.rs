use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_with_deprecated_tag4() {
    let content = r#"// @noLib: true
f({
    [|a|]/**/
    xyz: ` + "`" + `` + "`" + `,
});
declare function f(options: {
    /** @deprecated abc */
    abc?: number,
    xyz?: string
}): void;"#;
    let mut s = Session::new_for_test("completionsWithDeprecatedTag4", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

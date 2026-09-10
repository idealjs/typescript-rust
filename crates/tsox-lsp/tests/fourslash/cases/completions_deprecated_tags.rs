use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_deprecated_tags() {
    let content = r#"const o = {
    /** @deprecated */
    a: 1,
    b: 2,
    c: 3,
}
o./**/"#;
    let mut s = Session::new_for_test("completionsDeprecatedTags", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

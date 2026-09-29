use tsox_lsp::fourslash::{self, Session};


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
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

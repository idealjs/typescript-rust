use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_deprecated_tag5() {
    let content = r#"// @lib: es5
class Foo {
    /** @deprecated m */
    static m() {}
}
Foo./**/"#;
    let mut s = Session::new_for_test("completionsWithDeprecatedTag5", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

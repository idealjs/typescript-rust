use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_deprecated_tag6() {
    let content = r#"namespace Foo {
    /** @deprecated foo */
    export var foo: number;
}
Foo./**/"#;
    let mut s = Session::new_for_test("completionsWithDeprecatedTag6", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_deprecated_tag2() {
    let content = r#"/** @deprecated foo */
declare function foo<T>();
/** @deprecated foo<T> */
declare function foo<T>(x);

foo/**/"#;
    let mut s = Session::new_for_test("completionsWithDeprecatedTag2", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

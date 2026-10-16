use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_deprecated_tag3() {
    let content = r#"/** @deprecated foo */
declare function foo<T>();
/** ok */
declare function foo<T>(x);

foo/**/"#;
    let mut s = Session::new_for_test("completionsWithDeprecatedTag3", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

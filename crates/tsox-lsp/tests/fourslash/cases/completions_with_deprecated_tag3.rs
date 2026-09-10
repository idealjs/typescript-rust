use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_with_deprecated_tag3() {
    let content = r#"/** @deprecated foo */
declare function foo<T>();
/** ok */
declare function foo<T>(x);

foo/**/"#;
    let mut s = Session::new_for_test("completionsWithDeprecatedTag3", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

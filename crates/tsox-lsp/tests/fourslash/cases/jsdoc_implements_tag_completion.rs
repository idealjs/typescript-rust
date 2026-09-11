use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_implements_tag_completion() {
    let content = r#"// @lib: es5
/** @implements {/**/} */
class A {}"#;
    let mut s = Session::new_for_test("jsdocImplementsTagCompletion", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

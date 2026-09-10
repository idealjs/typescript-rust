use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn jsdoc_implements_tag_completion() {
    let content = r#"// @lib: es5
/** @implements {/**/} */
class A {}"#;
    let mut s = Session::new_for_test("jsdocImplementsTagCompletion", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

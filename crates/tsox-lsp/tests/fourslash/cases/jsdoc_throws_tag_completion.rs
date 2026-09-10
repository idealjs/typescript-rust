use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn jsdoc_throws_tag_completion() {
    let content = r#"// @lib: es5
/**
 * @throws {/**/} description
 */
function fn() {}"#;
    let mut s = Session::new_for_test("jsdocThrowsTagCompletion", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_throws_tag_completion() {
    let content = r#"// @lib: es5
/**
 * @throws {/**/} description
 */
function fn() {}"#;
    let mut s = Session::new_for_test("jsdocThrowsTagCompletion", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

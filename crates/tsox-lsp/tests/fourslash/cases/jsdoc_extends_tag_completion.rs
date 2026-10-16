use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_extends_tag_completion() {
    let content = r#"// @lib: es5
/** @extends {/**/} */
class A {}"#;
    let mut s = Session::new_for_test("jsdocExtendsTagCompletion", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

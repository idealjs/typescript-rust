use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_property_tag_completion() {
    let content = r#"// @lib: es5
/**
 * @typedef {Object} Foo
 * @property {/**/}
 */"#;
    let mut s = Session::new_for_test("jsdocPropertyTagCompletion", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

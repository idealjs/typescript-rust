use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn jsdoc_param_tag_special_keywords() {
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: test.js
/**
 * @param {string} type
 */
function test(type) {
    type./**/
}"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

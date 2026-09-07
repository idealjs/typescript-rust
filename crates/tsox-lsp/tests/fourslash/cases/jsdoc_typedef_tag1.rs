use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn jsdoc_typedef_tag1() {
    let content = r#"// @lib: es2015
// @allowNonTsExtensions: true
// @Filename: jsdocCompletion_typedef.js
/**
 * @typedef {Object} MyType
 * @property {string} yes
 */
function foo() { }
/**
 * @param {MyType} my
 */
function a(my) {
    my.yes./*1*/
}"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

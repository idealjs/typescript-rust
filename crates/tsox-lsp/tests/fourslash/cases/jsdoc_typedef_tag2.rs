use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn jsdoc_typedef_tag2() {
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: jsdocCompletion_typedef.js
/**
 * @typedef {Object} A.B.MyType
 * @property {string} yes
 */
function foo() {}
/**
 * @param {A.B.MyType} my2
 */
function a(my2) {
    my2.yes./*1*/
}
/**
 * @param {MyType} my2
 */
function b(my2) {
    my2.yes./*2*/
}"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}

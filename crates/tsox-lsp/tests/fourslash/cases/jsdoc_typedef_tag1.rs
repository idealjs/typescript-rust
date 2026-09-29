use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("jsdocTypedefTag1", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["charAt"], &[]);
}

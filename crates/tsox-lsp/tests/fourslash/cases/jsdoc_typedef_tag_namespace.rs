use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn jsdoc_typedef_tag_namespace() {
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: jsdocCompletion_typedef.js
/**
 * @typedef {string | number} T.NumberLike
 * @typedef {{age: number}} T.People
 * @typedef {string | number} T.O.Q.NumberLike
 * @type {T.NumberLike}
 */
var x; x./*1*/;
/** @type {T.O.Q.NumberLike} */
var x1; x1./*2*/;
/** @type {T.People} */
var x1; x1./*3*/;"#;
    let mut s = Session::new_for_test("jsdocTypedefTagNamespace", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyCompletions(t, []string{"1", "3"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}

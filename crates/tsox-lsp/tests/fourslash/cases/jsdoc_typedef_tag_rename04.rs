use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_typedef_tag_rename04() {
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: jsDocTypedef_form2.js

function test1() {
   /** @typedef {(string | number)} NumberLike */

   /** @type {/*1*/NumberLike} */
   var numberLike;
}
function test2() {
   /** @typedef {(string | number)} NumberLike2 */

   /** @type {NumberLike2} */
   var n/*2*/umberLike2;
}"#;
    let mut s = Session::new_for_test("jsdocTypedefTagRename04", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyQuickInfoExists(t)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "111");
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyQuickInfoExists(t)
}

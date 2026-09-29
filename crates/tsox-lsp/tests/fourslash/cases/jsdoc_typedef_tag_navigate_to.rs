use tsox_lsp::fourslash::Session;


#[test]
fn jsdoc_typedef_tag_navigate_to() {
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: jsDocTypedef_form2.js

/** @typedef {(string | number)} NumberLike */
/** @typedef {(string | number | string[])} */
var NumberLike2;

/** @type {/*1*/NumberLike} */
var numberLike;"#;
    let _s = Session::new_for_test("jsdocTypedefTagNavigateTo", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}

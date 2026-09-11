use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_callback_tag_rename01() {
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: jsDocCallback.js

/**
 * [|@callback [|{| "contextRangeIndex": 0 |}FooCallback|]
 * @param {string} eventName - Rename should work
 |]*/

/** @type {/*1*/[|FooCallback|]} */
var t;"#;
    let mut s = Session::new_for_test("jsdocCallbackTagRename01", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1])
}

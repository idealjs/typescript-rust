use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn jsdoc_typedef_tag_rename03() {
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: jsDocTypedef_form3.js

/**
 * [|@typedef /*1*/[|{| "contextRangeIndex": 0 |}Person|]
 * @type {Object}
 * @property {number} age
 * @property {string} name
 |]*/

/** @type {/*2*/[|Person|]} */
var person;"#;
    let mut s = Session::new_for_test("jsdocTypedefTagRename03", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_file(&mut s, "jsDocTypedef_form3.js");
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.GetRangesByText().Get("Person"))...)
}

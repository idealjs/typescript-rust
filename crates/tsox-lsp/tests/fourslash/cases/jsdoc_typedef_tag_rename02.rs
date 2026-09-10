use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn jsdoc_typedef_tag_rename02() {
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: jsDocTypedef_form2.js

/** [|@typedef {(string | number)} [|{| "contextRangeIndex": 0 |}NumberLike|]|] */

/** @type {[|NumberLike|]} */
var numberLike;"#;
    let mut s = Session::new_for_test("jsdocTypedefTagRename02", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.Ranges()[1:])...)
}

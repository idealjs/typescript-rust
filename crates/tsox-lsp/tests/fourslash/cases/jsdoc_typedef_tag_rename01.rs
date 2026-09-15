use tsox_lsp::fourslash::Session;


#[test]
fn jsdoc_typedef_tag_rename01() {
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: jsDocTypedef_form1.js

/** @typedef {(string | number)} */
[|var [|{| "contextRangeIndex": 0 |}NumberLike|];|]

[|NumberLike|] = 10;

/** @type {[|NumberLike|]} */
var numberLike;"#;
    let _s = Session::new_for_test("jsdocTypedefTagRename01", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.Ranges()[1:])...)
}

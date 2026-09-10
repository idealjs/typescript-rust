use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_import_type_js4() {
    let content = r#"// @module: commonjs
// @allowJs: true
// @checkJs: true
// @Filename: /a.js
/**
 * @callback /**/A
 * @param {unknown} response
 */

module.exports = {};
// @Filename: /b.js
/** @typedef {import("./a").A} A */"#;
    let mut s = Session::new_for_test("findAllRefs_importType_js4", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}

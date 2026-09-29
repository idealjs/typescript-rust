use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_js_doc_type_literal() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: foo.js
/**
 * @param {object} o - very important!
 * @param {string} o.x - a thing, its ok
 * @param {number} o.y - another thing
 * @param {Object} o.nested - very nested
 * @param {boolean} o.nested./*1*/great - much greatness
 * @param {number} o.nested.times - twice? probably!??
 */
 function f(o) { return o.nested./*2*/great; }"#;
    let _s = Session::new_for_test("findAllReferencesJsDocTypeLiteral", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}

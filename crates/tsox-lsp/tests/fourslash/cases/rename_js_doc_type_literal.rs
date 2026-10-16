use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_js_doc_type_literal() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @filename: /a.js
/**
 * @param {Object} options
 * @param {string} options.foo
 * @param {number} options.bar
 */
function foo(/**/options) {}"#;
    let mut s = Session::new_for_test("renameJsDocTypeLiteral", content);
    fourslash::go_to_file(&mut s, "/a.js");
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}

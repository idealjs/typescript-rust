use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_js_doc_type_def_js() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/** /*1*/@typedef {number} /*2*/T */

/**
 * @return {/*3*/T}
 */
function f(obj) { return 0; }

/**
 * @return {/*4*/T}
 */
function f2(obj) { return 0; }"#;
    let mut s = Session::new_for_test("findAllRefsJsDocTypeDef_js", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}

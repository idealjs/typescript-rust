use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_js_doc_template_tag_function_js() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/**
 * @template /*1*/T
 * @return {/*2*/T}
 */
function f() {}"#;
    let mut s = Session::new_for_test("findAllRefsJsDocTemplateTag_function_js", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}

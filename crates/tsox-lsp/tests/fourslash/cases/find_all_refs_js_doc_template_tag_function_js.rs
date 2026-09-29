use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_js_doc_template_tag_function_js() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/**
 * @template /*1*/T
 * @return {/*2*/T}
 */
function f() {}"#;
    let _s = Session::new_for_test("findAllRefsJsDocTemplateTag_function_js", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}

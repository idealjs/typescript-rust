use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_js_doc_template_tag_function() {
    let content = r#"/** @template /*1*/T */
function f</*2*/T>() {}"#;
    let mut s = Session::new_for_test("findAllRefsJsDocTemplateTag_function", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}

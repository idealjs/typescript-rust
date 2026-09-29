use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_js_doc_template_tag_class() {
    let content = r#"/** @template /*1*/T */
class C</*2*/T> {}"#;
    let _s = Session::new_for_test("findAllRefsJsDocTemplateTag_class", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}

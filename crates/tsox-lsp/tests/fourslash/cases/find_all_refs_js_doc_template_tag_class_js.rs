use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_js_doc_template_tag_class_js() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/** @template /*1*/T */
class C {
    constructor() {
        /** @type {/*2*/T} */
        this.x = null;
    }
}"#;
    let mut s = Session::new_for_test("findAllRefsJsDocTemplateTag_class_js", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}

use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_js_doc_tags2() {
    let content = r#"// @Filename: quickInfoJsDocTags2.ts
/** Doc   */
const /**/x = 0;"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "const x: 0", "Doc");
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_js_doc_this_tag() {
    let content = r#"// @strict: true
// @filename: /a.ts
/** @this {number} */
function f/**/() {
    this
}"#;
    let mut s = Session::new_for_test("quickInfoJsDocThisTag", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}

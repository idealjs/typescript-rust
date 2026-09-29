use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_js_doc_comment_with_no_tags() {
    let content = r#"/** Test */
export const Test = {}"#;
    let _s = Session::new_for_test("navigationBarJsDocCommentWithNoTags", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}

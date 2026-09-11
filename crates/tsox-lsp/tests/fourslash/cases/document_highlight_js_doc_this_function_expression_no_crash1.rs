use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlight_js_doc_this_function_expression_parameter() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/**@this{A}*/x=function(/*m*/a){};"#;
    let mut s = Session::new_for_test("documentHighlightJSDocThisFunctionExpressionParameter", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "m")
}

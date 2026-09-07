use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_js_doc_this_function_expression_parameter() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/**@this{A}*/x=function(/*m*/a){};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "m")
}

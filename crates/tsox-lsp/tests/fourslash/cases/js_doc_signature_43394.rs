use tsox_lsp::fourslash::Session;


#[test]
fn js_doc_signature_43394() {
    let content = r#"/**
 * @typedef {Object} Foo
 * @property {number} ...
 * /**/@typedef {number} Bar
 */"#;
    let _s = Session::new_for_test("jsDocSignature_43394", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}

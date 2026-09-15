use tsox_lsp::fourslash::Session;


#[test]
fn annotate_with_type_from_js_doc2() {
    let content = r#"// @Filename: test123.ts
/** @type {number} */
var [|x|]: string;"#;
    let _s = Session::new_for_test("annotateWithTypeFromJSDoc2", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}

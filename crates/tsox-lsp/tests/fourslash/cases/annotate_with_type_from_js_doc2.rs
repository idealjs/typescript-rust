use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn annotate_with_type_from_js_doc2() {
    let content = r#"// @Filename: test123.ts
/** @type {number} */
var [|x|]: string;"#;
    let mut s = Session::new_for_test("annotateWithTypeFromJSDoc2", content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, nil)
}

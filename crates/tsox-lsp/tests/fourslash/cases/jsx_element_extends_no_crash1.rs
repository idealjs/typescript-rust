use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsx_element_extends_no_crash1() {
    let content = r#"// @filename: index.tsx
<const T extends/>"#;
    let mut s = Session::new_for_test("jsxElementExtendsNoCrash1", content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, nil)
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsx_element_extends_no_crash3() {
    let content = r#"// @filename: index.tsx
<T extends /=>"#;
    let mut s = Session::new_for_test("jsxElementExtendsNoCrash3", content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, nil)
}

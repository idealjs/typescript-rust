use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsx_element_extends_no_crash1() {
    let content = r#"// @filename: index.tsx
<const T extends/>"#;
    let mut s = Session::new_for_test("jsxElementExtendsNoCrash1", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}

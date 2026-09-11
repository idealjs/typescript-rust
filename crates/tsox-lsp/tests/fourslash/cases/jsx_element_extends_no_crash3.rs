use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsx_element_extends_no_crash3() {
    let content = r#"// @filename: index.tsx
<T extends /=>"#;
    let mut s = Session::new_for_test("jsxElementExtendsNoCrash3", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}

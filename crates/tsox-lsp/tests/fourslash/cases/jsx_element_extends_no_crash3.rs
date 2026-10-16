use tsox_lsp::fourslash::Session;


#[test]
fn jsx_element_extends_no_crash3() {
    let content = r#"// @filename: index.tsx
<T extends /=>"#;
    let _s = Session::new_for_test("jsxElementExtendsNoCrash3", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}

use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_function_like_property_assignments() {
    let content = r#"var functions = {
    a: 0,
    b: function () { },
    c: function x() { },
    d: () => { },
    e: y(),
    f() { }
};"#;
    let _s = Session::new_for_test("navigationBarFunctionLikePropertyAssignments", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}

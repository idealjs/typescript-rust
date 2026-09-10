use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
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
    let mut s = Session::new_for_test("navigationBarFunctionLikePropertyAssignments", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}

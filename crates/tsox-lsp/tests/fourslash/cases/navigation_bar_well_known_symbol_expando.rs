use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_well_known_symbol_expando() {
    let content = r#"function f() {}
f[Symbol.iterator] = function() {}"#;
    let _s = Session::new_for_test("navigationBarWellKnownSymbolExpando", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}

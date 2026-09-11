use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_well_known_symbol_expando() {
    let content = r#"function f() {}
f[Symbol.iterator] = function() {}"#;
    let mut s = Session::new_for_test("navigationBarWellKnownSymbolExpando", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn navbar_const() {
    let content = r#"const c = 0;"#;
    let mut s = Session::new_for_test("navbar_const", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}

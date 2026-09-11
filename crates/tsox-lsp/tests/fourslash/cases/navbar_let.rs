use tsox_lsp::fourslash::{self, Session};


#[test]
fn navbar_let() {
    let content = r#"let c = 0;"#;
    let mut s = Session::new_for_test("navbar_let", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}

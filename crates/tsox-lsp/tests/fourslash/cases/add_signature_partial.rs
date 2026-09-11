use tsox_lsp::fourslash::{self, Session};


#[test]
fn add_signature_partial() {
    let content = r#""#;
    let mut s = Session::new_for_test("addSignaturePartial", content);
    // TODO: f.Insert(t, "interface Number { toFixed")
}

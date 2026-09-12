use tsox_lsp::fourslash::{self, Session};


#[test]
fn add_signature_partial() {
    let content = r#""#;
    let mut s = Session::new_for_test("addSignaturePartial", content);
    fourslash::insert(&mut s, "interface Number { toFixed");
    fourslash::insert(&mut s, "(");
    // TODO: }
}

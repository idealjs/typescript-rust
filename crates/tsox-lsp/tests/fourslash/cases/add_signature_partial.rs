use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.Insert(t, 'interface Number { toFixed')"]
#[test]
fn add_signature_partial() {
    let content = r#""#;
    let mut s = Session::new(content);
    // TODO: f.Insert(t, "interface Number { toFixed")
}

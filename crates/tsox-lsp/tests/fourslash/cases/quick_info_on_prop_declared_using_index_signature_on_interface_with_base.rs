use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_prop_declared_using_index_signature_on_interface_with_base() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface P {}
interface B extends P {
  [k: string]: number;
}
declare const b: B;
b.t/*1*/est = 10;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(index) B[string]: number", "")
}

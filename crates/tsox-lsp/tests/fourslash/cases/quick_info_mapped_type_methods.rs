use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_mapped_type_methods() {
    let content = r#"type M = { [K in 'one']: any };
const x: M = {
  /**/one() {}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(property) one: any", "")
}

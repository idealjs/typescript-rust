use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_element_access_in_write_location4() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: true
interface Serializer {
  set value(v: string | number | boolean);
  get value(): string;
}
declare let box: Serializer;
box['value'/*1*/] = true;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) Serializer.value: string | number | boolean", "")
}

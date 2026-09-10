use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_property_access_in_write_location5() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: true
interface Serializer {
  set value(v: string | number);
  get value(): string;
}
declare let box: Serializer;
box.value/*1*/ += 10;"#;
    let mut s = Session::new_for_test("quickInfoOnPropertyAccessInWriteLocation5", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) Serializer.value: string | number", "");
}

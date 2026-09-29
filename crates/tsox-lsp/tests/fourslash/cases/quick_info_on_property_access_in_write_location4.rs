use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_property_access_in_write_location4() {
    let content = r#"// @strict: true
interface Serializer {
  set value(v: string | number | boolean);
  get value(): string;
}
declare let box: Serializer;
box.value/*1*/ = true;"#;
    let mut s = Session::new_for_test("quickInfoOnPropertyAccessInWriteLocation4", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) Serializer.value: string | number | boolean", "");
}

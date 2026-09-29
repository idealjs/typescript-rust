use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_element_access_in_write_location5() {
    let content = r#"// @strict: true
interface Serializer {
  set value(v: string | number);
  get value(): string;
}
declare let box: Serializer;
box['value'/*1*/] += 10;"#;
    let mut s = Session::new_for_test("quickInfoOnElementAccessInWriteLocation5", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) Serializer.value: string | number", "");
}

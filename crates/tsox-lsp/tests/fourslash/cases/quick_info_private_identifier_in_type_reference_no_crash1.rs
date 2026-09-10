use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_private_identifier_in_type_reference_no_crash1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @target: esnext
class Foo {
  #prop: string = "";

  method() {
    const test: Foo.#prop/*1*/ = "";
  }
}"#;
    let mut s = Session::new_for_test("quickInfoPrivateIdentifierInTypeReferenceNoCrash1", content);
    fourslash::verify_quick_info_at(&mut s, "1", "", "");
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_mapped_type_methods() {
    let content = r#"type M = { [K in 'one']: any };
const x: M = {
  /**/one() {}
}"#;
    let mut s = Session::new_for_test("quickInfoMappedTypeMethods", content);
    fourslash::verify_quick_info_at(&mut s, "", "(property) one: any", "");
}

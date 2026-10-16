use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_element_access_in_write_location1() {
    let content = r#"// @strict: true
// @exactOptionalPropertyTypes: true
declare const xx: { prop?: number };
xx['prop'/*1*/] = 1;"#;
    let mut s = Session::new_for_test("quickInfoOnElementAccessInWriteLocation1", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) prop?: number", "");
}

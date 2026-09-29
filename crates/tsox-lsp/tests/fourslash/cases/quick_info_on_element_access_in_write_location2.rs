use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_element_access_in_write_location2() {
    let content = r#"// @strict: true
// @exactOptionalPropertyTypes: true
declare const xx: { prop?: number };
xx['prop'/*1*/] += 1;"#;
    let mut s = Session::new_for_test("quickInfoOnElementAccessInWriteLocation2", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) prop?: number", "");
}

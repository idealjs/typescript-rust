use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_on_property_access_in_write_location1() {
    let content = r#"// @strict: true
// @exactOptionalPropertyTypes: true
declare const xx: { prop?: number };
xx.prop/*1*/ = 1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) prop?: number", "")
}

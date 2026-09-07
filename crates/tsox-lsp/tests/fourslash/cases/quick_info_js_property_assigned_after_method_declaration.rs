use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_js_property_assigned_after_method_declaration() {
    let content = r#"// @noLib: true
// @allowJs: true
// @noImplicitThis: true
// @Filename: /a.js
const o = {
    test/*1*/() {
        this./*2*/test = 0;
    }
};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(method) test(): void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(method) test(): void", "")
}

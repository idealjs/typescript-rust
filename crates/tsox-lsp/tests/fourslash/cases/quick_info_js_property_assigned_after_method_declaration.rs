use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("quickInfoJsPropertyAssignedAfterMethodDeclaration", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(method) test(): void", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(method) test(): void", "");
}

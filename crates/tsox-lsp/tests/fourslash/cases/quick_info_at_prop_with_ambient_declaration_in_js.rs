use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_at_prop_with_ambient_declaration_in_js() {
    let content = r#"// @allowJs: true
// @filename: /a.js
class C {
    constructor() {
        this.prop = "";
    }
    declare prop: string;
    method() {
        this.prop.foo/**/
    }
}"#;
    let mut s = Session::new_for_test("quickInfoAtPropWithAmbientDeclarationInJs", content);
    // TODO: f.VerifyBaselineHover(t)
}

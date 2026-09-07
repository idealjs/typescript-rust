use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_js_export() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: a.js
// @allowJs: true
/**
 * @enum {string}
 */
const testString = {
    one: "1",
    two: "2"
};

export { test/**/String };"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "",
        "(alias) type testString = string\n(alias) const testString: {\n    one: string;\n    two: string;\n}\nexport testString",
        "",
    );
}

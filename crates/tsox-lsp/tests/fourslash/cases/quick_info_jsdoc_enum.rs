use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_jsdoc_enum() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @noLib: true
// @Filename: /a.js
/**
 * Doc
 * @enum {number}
 */
const E = {
    A: 0,
}

/** @type {/*type*/E} */
const x = /*value*/E.A;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "type", "type E = number", "Doc")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "value", "const E: {\n    A: number;\n}", "Doc")
}

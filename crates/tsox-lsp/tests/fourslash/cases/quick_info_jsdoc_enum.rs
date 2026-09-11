use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_jsdoc_enum() {
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
    let mut s = Session::new_for_test("quickInfoJsdocEnum", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::verify_quick_info_at(&mut s, "type", "type E = number", "Doc");
    fourslash::verify_quick_info_at(&mut s, "value", "const E: {\n    A: number;\n}", "Doc");
}

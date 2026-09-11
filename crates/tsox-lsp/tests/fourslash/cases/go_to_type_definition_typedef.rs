use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_type_definition_typedef() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/**
 * /*def*/@typedef {object} I
 * @property {number} x
 */

/** @type {I} */
const /*ref*/i = { x: 0 };"#;
    let mut s = Session::new_for_test("goToTypeDefinition_typedef", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "ref")
}

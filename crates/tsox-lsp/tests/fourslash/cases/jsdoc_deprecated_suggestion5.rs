use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion5() {
    let content = r#"// @checkJs: true
// @allowJs: true
// @Filename: jsdocDeprecated_suggestion5.js
/** @typedef {{ email: string, nickName?: string }} U2 */
/** @type {U2} */
const u2 = { email: "" }
/**
 * @callback K
 * @param {any} ctx
 * @return {void}
 */
/** @type {K} */
const cc = _k => {}
/** @enum {number} */
const DOOM = { e: 1, m: 1 }
/** @type {DOOM} */
const kneeDeep = DOOM.e"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion5", content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, nil)
}

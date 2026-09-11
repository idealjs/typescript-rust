use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_verbosity_js_doc_namespaced_typedef() {
    let content = r#"
// @allowJs: true
// @checkJs: true
// @Filename: /index.js
// Namespaced typedef
/** @typedef {string} /*ns*/NS./*t*/T */

// Namespaced typedef aliased to qualified namespaced typedef.
/** @typedef {NS.T} NS./*u*/U */

// Namespaced typedef aliased to implicitly-resolved typedef.
/** @typedef {U} NS./*v*/V */
"#;
    let mut s = Session::new_for_test("quickInfoVerbosityJSDocNamespacedTypedef", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{
}

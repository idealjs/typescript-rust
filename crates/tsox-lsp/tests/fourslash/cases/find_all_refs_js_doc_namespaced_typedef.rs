use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_js_doc_namespaced_typedef() {
    let content = r#"
// @allowJs: true
// @checkJs: true
// @Filename: /index.js
// Namespaced typedef
/** @typedef {string} [|NS|].[|T|] */

// Namespaced typedef aliased to qualified namespaced typedef.
/** @typedef {NS.T} NS.[|U|] */

// Namespaced typedef aliased to implicitly-resolved typedef.
/** @typedef {U} NS.[|V|] */
"#;
    let mut s = Session::new_for_test("findAllRefsJSDocNamespacedTypedef", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t)
}

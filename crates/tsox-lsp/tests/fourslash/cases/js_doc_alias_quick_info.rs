use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn js_doc_alias_quick_info() {
    let content = r#"// @Filename: /jsDocAliasQuickInfo.ts
/**
 * Comment
 * @type {number}
 */
export /*1*/default 10;
// @Filename: /test.ts
export { /*2*/default as /*3*/test } from "./jsDocAliasQuickInfo";"#;
    let mut s = Session::new_for_test("jsDocAliasQuickInfo", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}

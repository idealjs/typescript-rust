use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyQuickInfoIs"]
#[test]
fn quick_info_js_doc_link_backticks() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @checkJs: true
// @strict: true
// @Filename: jsdocParseMatchingBackticks.js
/**
 * ` + "`" + `{@link foo}` + "`" + ` initial at-param is OK in title comment
 * @param {string} x hi there ` + "`" + `{@link foo}` + "`" + `
 */
export function f(x) {
    return x/*x*/
}
f/*f*/"#;
    let mut s = Session::new_for_test("quickInfoJSDocLinkBackticks", content);
    fourslash::go_to_marker(&mut s, "f");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "function f(x: string): string", "`{@link foo}` initial at-param is OK in tit
    fourslash::go_to_marker(&mut s, "x");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(parameter) x: string", "hi there `{@link foo}`")
}

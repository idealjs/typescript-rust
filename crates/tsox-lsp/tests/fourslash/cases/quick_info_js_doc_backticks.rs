use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_js_doc_backticks() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @checkJs: true
// @strict: true
// @Filename: jsdocParseMatchingBackticks.js
/**
 * `@param` initial at-param is OK in title comment
 * @param {string} x hi there `@param`
 * @param {string} y hi there `@ * param
 *                   this is the margin
 */
export function f(x, y) {
    return x/*x*/ + y/*y*/
}
f/*f*/"#;
    let mut s = Session::new_for_test("quickInfoJSDocBackticks", content);
    fourslash::go_to_marker(&mut s, "f");
    // TODO: f.VerifyQuickInfoIs(t, "function f(x: string, y: string): string", "`@param` initial at-param is OK 
    fourslash::go_to_marker(&mut s, "x");
    // TODO: f.VerifyQuickInfoIs(t, "(parameter) x: string", "hi there `@param`")
    fourslash::go_to_marker(&mut s, "y");
    // TODO: f.VerifyQuickInfoIs(t, "(parameter) y: string", "hi there `@ * param\nthis is the margin")
}

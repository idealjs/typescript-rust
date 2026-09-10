use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_js_doc_param_with_invalid_tag_in_comment() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/**
 * @param {string} x Checks @-rule here
 * @param {string} a see @foo*bar here
 * @param {string} b see @test(something) here
 * @param {string} c see @*not-ident here
 * @param {string} d see @(paren) here
 */
function /*fn*/foo(/**/x, /*a*/a, /*b*/b, /*c*/c, /*d*/d) {}
"#;
    let mut s = Session::new_for_test("quickInfoJSDocParamWithInvalidTagInComment", content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "fn", "function foo(x: string, a: string, b: string, c: string, d: string): v
    fourslash::verify_quick_info_at(&mut s, "", "(parameter) x: string", "Checks @-rule here");
    fourslash::verify_quick_info_at(&mut s, "a", "(parameter) a: string", "see");
    fourslash::verify_quick_info_at(&mut s, "b", "(parameter) b: string", "see");
    fourslash::verify_quick_info_at(&mut s, "c", "(parameter) c: string", "see @*not-ident here");
    fourslash::verify_quick_info_at(&mut s, "d", "(parameter) d: string", "see @(paren) here");
}

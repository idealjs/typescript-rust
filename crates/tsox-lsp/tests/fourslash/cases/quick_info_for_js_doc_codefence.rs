use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_for_js_doc_codefence() {
    let content = r#"/**
 * @example
 * ```
 * 1 + 2
 * ```
 */
function fo/*1*/o() {
    return '2';
}
/**
 * @example
 * ``
 * 1 + 2
 * `
 */
function bo/*2*/o() {
    return '2';
}"#;
    let _s = Session::new_for_test("quickInfoForJSDocCodefence", content);
    // TODO: f.VerifyBaselineHover(t)
}

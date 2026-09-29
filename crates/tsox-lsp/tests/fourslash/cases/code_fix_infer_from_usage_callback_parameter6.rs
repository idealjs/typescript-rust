use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_infer_from_usage_callback_parameter6() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @noImplicitAny: false
// @filename: /foo.js
const foo = [(/** @type {number} */ x) => x + 1];"#;
    let _s = Session::new_for_test("codeFixInferFromUsageCallbackParameter6", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}

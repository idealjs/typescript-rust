use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_crash1() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: foo.js
/**
 * @param {function(string): boolean} f
 */
function doThing(f) {
    f(100)
}"#;
    let _s = Session::new_for_test("inlayHintsCrash1", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

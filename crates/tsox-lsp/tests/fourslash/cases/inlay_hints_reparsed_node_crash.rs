use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_reparsed_node_crash() {
    let content = r#"
// @allowJs: true
// @checkJs: true

// @Filename: /a.js
module.exports = function () {
  return 1;
};
"#;
    let _s = Session::new_for_test("inlayHintsReparsedNodeCrash", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

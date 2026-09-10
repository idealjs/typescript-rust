use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
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
    let mut s = Session::new_for_test("inlayHintsReparsedNodeCrash", content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

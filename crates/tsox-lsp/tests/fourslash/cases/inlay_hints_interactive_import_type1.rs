use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_interactive_import_type1() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: /a.js
module.exports.a = 1
// @Filename: /b.js
const a = require('./a');"#;
    let mut s = Session::new_for_test("inlayHintsInteractiveImportType1", content);
    fourslash::go_to_file(&mut s, "/b.js");
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

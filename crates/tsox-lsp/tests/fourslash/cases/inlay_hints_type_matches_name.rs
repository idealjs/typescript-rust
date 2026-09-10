use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_type_matches_name() {
    let content = r#"type Client = {};
function getClient(): Client { return {}; };
const client = getClient();"#;
    let mut s = Session::new_for_test("inlayHintsTypeMatchesName", content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

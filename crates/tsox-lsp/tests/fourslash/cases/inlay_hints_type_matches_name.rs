use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_type_matches_name() {
    let content = r#"type Client = {};
function getClient(): Client { return {}; };
const client = getClient();"#;
    let _s = Session::new_for_test("inlayHintsTypeMatchesName", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

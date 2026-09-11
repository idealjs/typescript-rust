use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_identifier_location() {
    let content = r#"interface Foo {}
const p = (a: Foo[]) => a;"#;
    let mut s = Session::new_for_test("inlayHintsIdentifierLocation", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

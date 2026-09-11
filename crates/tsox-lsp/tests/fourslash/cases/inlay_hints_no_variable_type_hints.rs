use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_no_variable_type_hints() {
    let content = r#"const a = 123;"#;
    let mut s = Session::new_for_test("inlayHintsNoVariableTypeHints", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

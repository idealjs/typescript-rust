use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_enum_member_value() {
    let content = r#"enum E {
    A,
    AA,
    B = 10,
    BB,
    C = 'C',
}"#;
    let _s = Session::new_for_test("inlayHintsEnumMemberValue", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}

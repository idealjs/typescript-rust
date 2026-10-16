use tsox_lsp::fourslash::Session;


#[test]
fn rename_numerical_index_single_quoted() {
    let content = r#"const foo = { [|0|]: true };
foo[[|0|]];"#;
    let _s = Session::new_for_test("renameNumericalIndexSingleQuoted", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, &lsutil.UserPreferences{QuotePreference: lsutil.QuotePrefe
}

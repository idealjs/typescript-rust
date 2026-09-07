use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_numerical_index_single_quoted() {
    let content = r#"const foo = { [|0|]: true };
foo[[|0|]];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, &lsutil.UserPreferences{QuotePreference: lsutil.QuotePrefe
}

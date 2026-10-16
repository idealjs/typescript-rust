use tsox_lsp::fourslash::Session;


#[test]
fn rename_numerical_index() {
    let content = r#"const foo = { [|0|]: true };
foo[[|0|]];"#;
    let _s = Session::new_for_test("renameNumericalIndex", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "0")
}

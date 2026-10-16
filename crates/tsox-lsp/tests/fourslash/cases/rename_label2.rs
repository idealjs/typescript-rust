use tsox_lsp::fourslash::Session;


#[test]
fn rename_label2() {
    let content = r#"/**/foo: {
    break foo;
}"#;
    let _s = Session::new_for_test("renameLabel2", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}

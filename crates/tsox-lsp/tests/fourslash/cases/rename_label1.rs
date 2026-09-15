use tsox_lsp::fourslash::Session;


#[test]
fn rename_label1() {
    let content = r#"foo: {
    break /**/foo;
}"#;
    let _s = Session::new_for_test("renameLabel1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}

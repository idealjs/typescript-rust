use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_label2() {
    let content = r#"/**/foo: {
    break foo;
}"#;
    let mut s = Session::new_for_test("renameLabel2", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}

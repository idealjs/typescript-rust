use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_label1() {
    let content = r#"foo: {
    break /**/foo;
}"#;
    let mut s = Session::new_for_test("renameLabel1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}

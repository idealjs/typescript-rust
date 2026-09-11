use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_export_string_rename() {
    let content = r#"const foo = 123;
export { foo as /**/"bar" };"#;
    let mut s = Session::new_for_test("findAllRefsExportStringRename", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}

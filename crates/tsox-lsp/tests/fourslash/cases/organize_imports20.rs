use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports20() {
    let content = r#"const a = 1;
const b = 1;
export { a };
export { b };"#;
    let _s = Session::new_for_test("organizeImports20", content);
    // TODO: f.VerifyOrganizeImports(t,
}

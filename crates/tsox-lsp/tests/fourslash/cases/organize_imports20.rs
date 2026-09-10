use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports20() {
    let content = r#"const a = 1;
const b = 1;
export { a };
export { b };"#;
    let mut s = Session::new_for_test("organizeImports20", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

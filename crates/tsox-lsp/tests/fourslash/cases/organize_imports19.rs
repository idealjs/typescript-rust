use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports19() {
    let content = r#"const a = 1;
export { a };

const b = 1;
export { b };

const c = 1;
export { c };"#;
    let mut s = Session::new_for_test("organizeImports19", content);
    // TODO: f.VerifyOrganizeImports(t,
}

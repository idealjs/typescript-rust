use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_type2() {
    let content = r#"// @allowSyntheticDefaultImports: true
// @moduleResolution: bundler
// @noUnusedLocals: true
// @target: es2018
type A = string;
type B = string;
const C = "hello";
export { A, type B, C };"#;
    let mut s = Session::new_for_test("organizeImportsType2", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

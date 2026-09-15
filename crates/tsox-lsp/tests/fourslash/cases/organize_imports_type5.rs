use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports_type5() {
    let content = r#"import {
    d, 
    type d as D,
    type c,
    c as C,
    b,
    b as B,
    type A,
    a
} from './foo';
console.log(A, a, B, b, c, C, d, D);"#;
    let _s = Session::new_for_test("organizeImportsType5", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
}

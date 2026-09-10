use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_type3() {
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
    let mut s = Session::new_for_test("organizeImportsType3", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

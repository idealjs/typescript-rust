use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_unicode3() {
    let content = r#"import {
    B,
    À,
    A,
} from './foo';

console.log(A, À, B);"#;
    let mut s = Session::new_for_test("organizeImportsUnicode3", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

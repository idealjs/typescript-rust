use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports_unicode3() {
    let content = r#"import {
    B,
    À,
    A,
} from './foo';

console.log(A, À, B);"#;
    let _s = Session::new_for_test("organizeImportsUnicode3", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
}

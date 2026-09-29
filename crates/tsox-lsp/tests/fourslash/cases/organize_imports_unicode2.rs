use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports_unicode2() {
    let content = r#"import {
    a2,
    a100,
    a1,
} from './foo';

console.log(a1, a2, a100);"#;
    let _s = Session::new_for_test("organizeImportsUnicode2", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
}

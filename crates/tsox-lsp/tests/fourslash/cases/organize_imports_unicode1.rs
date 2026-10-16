use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports_unicode1() {
    let content = r#"import {
    Ab,
    _aB,
    aB,
    _Ab,
} from './foo';

console.log(_aB, _Ab, aB, Ab);"#;
    let _s = Session::new_for_test("organizeImportsUnicode1", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
}

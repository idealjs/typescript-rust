use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports_unicode4() {
    let content = r#"import {
    Ab,
    _aB,
    aB,
    _Ab,
} from './foo';

console.log(_aB, _Ab, aB, Ab);"#;
    let mut s = Session::new_for_test("organizeImportsUnicode4", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
}

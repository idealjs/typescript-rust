use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_unicode1() {
    let content = r#"import {
    Ab,
    _aB,
    aB,
    _Ab,
} from './foo';

console.log(_aB, _Ab, aB, Ab);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports22() {
    let content = r#"import {abc, Abc, bc, Bc} from 'b';
import {
  I,
  R,
  M,
} from 'a';
console.log(abc, Abc, bc, Bc, I, R, M);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

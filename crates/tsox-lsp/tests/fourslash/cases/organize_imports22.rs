use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports22() {
    let content = r#"import {abc, Abc, bc, Bc} from 'b';
import {
  I,
  R,
  M,
} from 'a';
console.log(abc, Abc, bc, Bc, I, R, M);"#;
    let mut s = Session::new_for_test("organizeImports22", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports23() {
    let content = r#"import {abc, Abc, type bc, type Bc} from 'b';
import {
  I,
  R,
  M,
} from 'a';
type x = bc | Bc;
console.log(abc, Abc, I, R, M);"#;
    let mut s = Session::new_for_test("organizeImports23", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
}

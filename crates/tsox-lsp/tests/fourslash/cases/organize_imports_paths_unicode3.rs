use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports_paths_unicode3() {
    let content = r#"import * as B from "./B";
import * as À from "./À";
import * as A from "./A";

console.log(A, À, B);"#;
    let mut s = Session::new_for_test("organizeImportsPathsUnicode3", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
}

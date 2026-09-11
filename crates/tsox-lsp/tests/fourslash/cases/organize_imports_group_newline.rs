use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports_group_newline() {
    let content = r#"import c from "C";

import d from "D";
import a from "A"; // not count
import b from "B";

console.log(a, b, c, d)"#;
    let mut s = Session::new_for_test("organizeImportsGroup_Newline", content);
    // TODO: f.VerifyOrganizeImports(t,
}

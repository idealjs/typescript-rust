use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_group_multi_newlines() {
    let content = r#"import c from "C";


import d from "D";
import a from "A";
import b from "B";

console.log(a, b, c, d)"#;
    let mut s = Session::new_for_test("organizeImportsGroup_MultiNewlines", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

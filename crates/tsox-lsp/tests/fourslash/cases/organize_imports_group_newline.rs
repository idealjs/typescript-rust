use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_group_newline() {
    let content = r#"import c from "C";

import d from "D";
import a from "A"; // not count
import b from "B";

console.log(a, b, c, d)"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

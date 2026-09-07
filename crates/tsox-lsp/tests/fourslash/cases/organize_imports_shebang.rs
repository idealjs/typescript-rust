use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_shebang_preserve_and_sort() {
    let content = r#"#!/usr/bin/env node
import Foo from "foo";
import Bar from "bar";

import Foobar from "foobar";

console.log(Foo, Bar, Foobar);"#;
    let mut s = Session::new(content);
    // TODO: f.VerifyOrganizeImports(
}

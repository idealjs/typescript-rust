use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports_shebang_preserve_and_sort() {
    let content = r#"#!/usr/bin/env node
import Foo from "foo";
import Bar from "bar";

import Foobar from "foobar";

console.log(Foo, Bar, Foobar);"#;
    let _s = Session::new_for_test("organizeImports_Shebang_PreserveAndSort", content);
    // TODO: f.VerifyOrganizeImports(
}

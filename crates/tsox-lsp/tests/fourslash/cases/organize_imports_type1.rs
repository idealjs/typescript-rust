use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_type1() {
    let content = r#"// @allowSyntheticDefaultImports: true
// @moduleResolution: bundler
// @noUnusedLocals: true
// @target: es2018
import { A } from "foo";
import { type B } from "foo";
import { C } from "foo";
import { type E } from "foo";
import { D } from "foo";

console.log(A, B, C, D, E);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

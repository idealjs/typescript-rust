use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("organizeImportsType1", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
}

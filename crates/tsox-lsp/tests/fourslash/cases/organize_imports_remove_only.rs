use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_remove_only() {
    let content = r#"import { c, b, a } from "foo";
import d, { e } from "bar";
import * as f from "baz";
import { g } from "foo";

export { g, e, b, c };"#;
    let mut s = Session::new_for_test("organizeImports_removeOnly", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

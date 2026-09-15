use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports_remove_only() {
    let content = r#"import { c, b, a } from "foo";
import d, { e } from "bar";
import * as f from "baz";
import { g } from "foo";

export { g, e, b, c };"#;
    let _s = Session::new_for_test("organizeImports_removeOnly", content);
    // TODO: f.VerifyOrganizeImports(t,
}

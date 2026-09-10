use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports7() {
    let content = r#"import * as something from "path"; /**
 * some comment here
 * and there
 */
import * as somethingElse from "anotherpath";

something;
somethingElse;"#;
    let mut s = Session::new_for_test("organizeImports7", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

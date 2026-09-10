use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports17() {
    let content = r#"import { Both } from "module-specifiers-unsorted";
import { aa, CaseInsensitively, sorted } from "aardvark";"#;
    let mut s = Session::new_for_test("organizeImports17", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

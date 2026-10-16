use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports17() {
    let content = r#"import { Both } from "module-specifiers-unsorted";
import { aa, CaseInsensitively, sorted } from "aardvark";"#;
    let _s = Session::new_for_test("organizeImports17", content);
    // TODO: f.VerifyOrganizeImports(t,
}

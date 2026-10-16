use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports_paths_unicode2() {
    let content = r#"import * as a2 from "./a2";
import * as a100 from "./a100";
import * as a1 from "./a1";

console.log(a1, a2, a100);"#;
    let _s = Session::new_for_test("organizeImportsPathsUnicode2", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
}

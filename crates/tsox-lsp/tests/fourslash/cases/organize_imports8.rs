use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports8() {
    let content = r#"import { foo as foo } from "foo";
foo;"#;
    let _s = Session::new_for_test("organizeImports8", content);
    // TODO: f.VerifyOrganizeImports(t,
}

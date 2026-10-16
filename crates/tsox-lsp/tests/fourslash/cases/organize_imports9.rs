use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports9() {
    let content = r#"import { a as a, b, c, d as d, e as e } from "foo";
a(b, d);"#;
    let _s = Session::new_for_test("organizeImports9", content);
    // TODO: f.VerifyOrganizeImports(t,
}

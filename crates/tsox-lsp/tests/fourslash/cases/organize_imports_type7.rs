use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports_type7() {
    let content = r#"import { a, type A, b } from "foo";
interface Use extends A {}
console.log(a, b);"#;
    let _s = Session::new_for_test("organizeImportsType7", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.ReplaceLine(t, 0, "import { a, type A, b } from \"foo1\";")
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.ReplaceLine(t, 0, "import { a, type A, b } from \"foo2\";")
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.ReplaceLine(t, 0, "import { a, type A, b } from \"foo3\";")
    // TODO: f.VerifyOrganizeImports(t,
}

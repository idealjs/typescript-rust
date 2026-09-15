use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports12() {
    let content = r#"// @allowJs: true
// @Filename: /test.js
declare export default class A {}
declare export { a, b };
declare export * from "foo";"#;
    let _s = Session::new_for_test("organizeImports12", content);
    // TODO: f.VerifyOrganizeImports(t,
}

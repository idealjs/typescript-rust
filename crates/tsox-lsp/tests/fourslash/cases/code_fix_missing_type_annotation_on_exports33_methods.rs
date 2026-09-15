use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports33_methods() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @Filename: /code.ts
export class Foo {
  m() {
  }
}"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports33_methods", content);
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Add return type 'void'"})
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

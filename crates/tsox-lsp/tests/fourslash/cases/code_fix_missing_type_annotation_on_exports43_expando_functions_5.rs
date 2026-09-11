use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports43_expando_functions_5() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
// @Filename: /code.ts
function foo(): void {}
// x already exists, so do not generate code for 'x'
foo.x = 1;
foo.y = 1;
namespace foo {
  export let x = 42;
}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports43_expando_functions_5", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

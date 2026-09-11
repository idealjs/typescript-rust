use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports5() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
const a = 42;
const b = 42;
export class C {
  get property() { return a + b; }
}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports5", content);
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Add return type 'number'"})
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

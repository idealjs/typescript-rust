use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports2() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
const a = 42;
const b = 43;
export function foo() { return a + b; }"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports2", content);
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Add return type 'number'"})
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

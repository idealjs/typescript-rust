use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports8() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function foo() {return 42;}
export const g = function () { return foo(); };"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports8", content);
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Add return type 'number'"})
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

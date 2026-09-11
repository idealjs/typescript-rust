use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports36_conditional_releative() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @Filename: /code.ts
const A = "A"
const B = "B"
export const AB = Math.random()? A: B;"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports36_conditional_releative", content);
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Add annotation of type '\"A\" | \"B\"'", "Add annotation of ty
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

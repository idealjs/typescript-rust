use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports20() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
export function foo () {
    return Symbol();
}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports20", content);
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, []string{"Add return type 'symbol'"})
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

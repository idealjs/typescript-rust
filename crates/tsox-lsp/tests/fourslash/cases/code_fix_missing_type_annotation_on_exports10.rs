use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports10() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function foo() {
    return { x: 1, y: 1 };
}
export default foo();"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports10", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

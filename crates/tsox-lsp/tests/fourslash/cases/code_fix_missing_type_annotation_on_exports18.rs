use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports18() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function foo() { return 42; }
export class A {
    readonly a = () => foo();
}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports18", content);
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, []string{"Add return type 'number'"})
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

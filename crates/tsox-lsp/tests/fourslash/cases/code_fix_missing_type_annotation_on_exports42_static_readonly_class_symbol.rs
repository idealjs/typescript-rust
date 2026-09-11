use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports42_static_readonly_class_symbol() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
// @Filename: /code.ts
class A {
    static readonly p1 = Symbol();
}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports42_static_readonly_class_symbol", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

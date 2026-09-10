use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports43_expando_functions_2() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
// @Filename: /code.ts
const foo = () => {}
foo/*a*/["a"] = "A";
foo["b"] = "C""#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports43_expando_functions_2", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

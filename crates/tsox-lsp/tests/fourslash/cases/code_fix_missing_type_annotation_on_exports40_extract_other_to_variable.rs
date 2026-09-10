use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports40_extract_other_to_variable() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
// @Filename: /code.ts
let c: string[] = [];
export let o = {
    p: Math.random() ? []: [
        ...c
    ]
}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports40_extract_other_to_variable", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports57_generics_doesnt_drop_trailing_unknown() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2015

let x: unknown;
export const s = new Set([x]);
"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports57_generics_doesnt_drop_trailing_unknown", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

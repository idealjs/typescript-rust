use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports41_no_computed_enum_members() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
// @Filename: /code.ts
enum E {
    A = "foo".length
}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports41_no_computed_enum_members", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}

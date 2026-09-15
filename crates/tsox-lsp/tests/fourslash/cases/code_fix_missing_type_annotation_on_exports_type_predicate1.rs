use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports_type_predicate1() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @filename: index.ts
export function isString(value: unknown) {
  return typeof value === "string";
}"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExportsTypePredicate1", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

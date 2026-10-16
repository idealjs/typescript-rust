use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports3() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
const a = 42;
const b = 42;
export class C {
  //making sure comments are not changed
  property =a+b; // comment should stay here
}"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports3", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

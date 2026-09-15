use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports9() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function foo( ){
    return 42;
}
const a = foo();
export = a;"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports9", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

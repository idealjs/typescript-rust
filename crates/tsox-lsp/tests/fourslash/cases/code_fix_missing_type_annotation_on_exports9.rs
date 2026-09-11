use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports9() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function foo( ){
    return 42;
}
const a = foo();
export = a;"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports9", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

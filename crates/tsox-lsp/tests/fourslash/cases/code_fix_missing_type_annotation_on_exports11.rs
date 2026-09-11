use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports11() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function mixin<T extends new (...a: any) => any>(ctor: T): T {
    return ctor;
}
class Point2D { x = 0; y = 0; }
export class Point3D extends mixin(Point2D) {  z = 0; }"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports11", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

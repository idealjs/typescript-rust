use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_missing_type_annotation_on_exports23_heritage_formatting() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function mixin<T extends new (...a: any) => any>(ctor: T): T {
    return ctor;
}
class Point2D { x = 0; y = 0; }
interface I{}
export class Point3D extends
    /** Base class */
    mixin(Point2D)
    // Test
    implements I
    {
              z = 0;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, []string{"Extract base class to variable"})
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

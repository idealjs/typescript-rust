use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_class_property_modifiers() {
    let content = r#"// @strict: false
abstract class A {
    abstract x: number;
    private y: number;
    protected z: number;
    public w: number;
    public useY() { this.y; }
}

class C implements A {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementClassPropertyModifiers", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

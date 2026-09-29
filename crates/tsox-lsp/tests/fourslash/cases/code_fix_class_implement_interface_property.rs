use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_property() {
    let content = r#"// @lib: es2017
enum E { a,b,c }
interface I {
    x: E;
    y: E.a
    z: symbol;
    w: object;
}
class C implements I {}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceProperty", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

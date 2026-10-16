use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_default_class() {
    let content = r#"interface I { x: number; }
export default class implements I {[| |]}"#;
    let _s = Session::new_for_test("codeFixClassImplementDefaultClass", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_all() {
    let content = r#"interface I { i(): void; }
interface J { j(): void; }
class C implements I, J {}
class D implements J {}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterface_all", content);
    // TODO: f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}

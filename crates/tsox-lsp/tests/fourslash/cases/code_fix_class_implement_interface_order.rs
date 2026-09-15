use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_order() {
    let content = r#"interface IFoo {
  bar(): void;
}

class Foo implements IFoo {
  private x = 1;
  constructor() { this.x = 2 }
}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterface_order", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

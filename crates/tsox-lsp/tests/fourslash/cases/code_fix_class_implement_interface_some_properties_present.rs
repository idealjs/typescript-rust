use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_some_properties_present() {
    let content = r#"// @strict: false

interface I {
    x: number;
    y: number;
    z: number & { __iBrand: any };
}

class C implements I {[|
   |]constructor(public x: number) { }
   y: number;
}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceSomePropertiesPresent", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

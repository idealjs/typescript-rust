use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_empty_type_literal() {
    let content = r#"
interface I {
    x: {};
}

class C implements I {[|
   |]constructor() { }
}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceEmptyTypeLiteral", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

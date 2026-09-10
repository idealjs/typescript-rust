use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
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
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

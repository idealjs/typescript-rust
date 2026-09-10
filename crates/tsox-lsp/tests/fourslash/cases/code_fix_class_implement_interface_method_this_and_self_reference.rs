use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_method_this_and_self_reference() {
    let content = r#"interface I {
    f(x: number, y: this): I
}

class C implements I {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMethodThisAndSelfReference", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

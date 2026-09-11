use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_method_type_predicate() {
    let content = r#"interface I {
    f(i: any): i is I;
    f(): this is I;
}

class C implements I {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMethodTypePredicate", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

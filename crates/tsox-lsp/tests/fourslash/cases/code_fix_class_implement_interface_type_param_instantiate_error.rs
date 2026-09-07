use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixAvailable"]
#[test]
fn code_fix_class_implement_interface_type_param_instantiate_error() {
    let content = r#"interface I<T extends string> {
   x: T;
}

class C implements I<number> { }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, []string{"Implement interface 'I<number>'"})
}

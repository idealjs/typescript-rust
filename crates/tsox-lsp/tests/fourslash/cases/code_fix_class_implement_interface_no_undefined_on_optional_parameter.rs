use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_no_undefined_on_optional_parameter() {
    let content = r#"interface IFoo {
    bar(x?: number | string): void;
}

class Foo implements IFoo {
}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterface_noUndefinedOnOptionalParameter", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

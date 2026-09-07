use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_class_implement_interface_duplicate_member2() {
    let content = r#"// @strict: false
interface I1 {
    x: number;
}
interface I2 {
    x: number;
}

class C implements I1,I2 {
    x: number;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}

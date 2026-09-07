use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_multiple_members_and_punctuation() {
    let content = r#"interface I1 {
    x: number,
    y: number
    z: number;
    f(): number,
    g(): any
    h();
}

class C1 implements I1 {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

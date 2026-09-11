use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMultipleMembersAndPunctuation", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

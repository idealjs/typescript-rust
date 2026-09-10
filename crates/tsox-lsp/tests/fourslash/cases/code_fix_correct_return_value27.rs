use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_correct_return_value27() {
    let content = r#"const a: ((() => number) | (() => undefined)) = () => { "" }"#;
    let mut s = Session::new_for_test("codeFixCorrectReturnValue27", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}

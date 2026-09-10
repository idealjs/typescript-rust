use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_correct_return_value5() {
    let content = r#"function Foo (): void {
    undefined
}"#;
    let mut s = Session::new_for_test("codeFixCorrectReturnValue5", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}

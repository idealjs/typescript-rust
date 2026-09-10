use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_correct_return_value6() {
    let content = r#"function Foo (): undefined {
    undefined
}"#;
    let mut s = Session::new_for_test("codeFixCorrectReturnValue6", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}

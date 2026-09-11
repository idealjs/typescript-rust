use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_correct_return_value5() {
    let content = r#"function Foo (): void {
    undefined
}"#;
    let mut s = Session::new_for_test("codeFixCorrectReturnValue5", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}

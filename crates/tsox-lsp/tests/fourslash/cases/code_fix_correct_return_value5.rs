use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_correct_return_value5() {
    let content = r#"function Foo (): void {
    undefined
}"#;
    let _s = Session::new_for_test("codeFixCorrectReturnValue5", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}

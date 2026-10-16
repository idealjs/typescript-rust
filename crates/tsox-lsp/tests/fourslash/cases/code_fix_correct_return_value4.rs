use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_correct_return_value4() {
    let content = r#"function Foo (): any {
    1
}"#;
    let _s = Session::new_for_test("codeFixCorrectReturnValue4", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}

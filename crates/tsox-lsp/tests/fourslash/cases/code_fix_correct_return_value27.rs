use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_correct_return_value27() {
    let content = r#"const a: ((() => number) | (() => undefined)) = () => { "" }"#;
    let mut s = Session::new_for_test("codeFixCorrectReturnValue27", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}

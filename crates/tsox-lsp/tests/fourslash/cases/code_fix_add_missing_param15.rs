use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_add_missing_param15() {
    let content = r#"function f(a: number, b: number) {}
f();"#;
    let _s = Session::new_for_test("codeFixAddMissingParam15", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "addMissingParam")
}

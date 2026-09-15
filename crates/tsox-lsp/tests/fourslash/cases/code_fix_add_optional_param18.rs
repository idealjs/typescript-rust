use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_add_optional_param18() {
    let content = r#"[|function f(a: number, c: string) {}|]
f(1, 1, "");"#;
    let _s = Session::new_for_test("codeFixAddOptionalParam18", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "addOptionalParam")
}

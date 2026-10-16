use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_add_optional_param14() {
    let content = r#"function f(a: string): string;
function f(a: string, b: number): string;
function f(a: string, b?: number): string {
    return "";
}
f("", "", 1);"#;
    let _s = Session::new_for_test("codeFixAddOptionalParam14", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "addOptionalParam")
}

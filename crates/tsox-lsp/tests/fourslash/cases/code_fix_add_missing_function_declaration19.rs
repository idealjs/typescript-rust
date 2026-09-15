use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_add_missing_function_declaration19() {
    let content = r#"declare function f(x: number): any;
f(foo);"#;
    let _s = Session::new_for_test("codeFixAddMissingFunctionDeclaration19", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixMissingFunctionDeclaration")
}

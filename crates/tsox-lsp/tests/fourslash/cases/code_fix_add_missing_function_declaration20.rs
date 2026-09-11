use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_add_missing_function_declaration20() {
    let content = r#"const a = {
   b: { f(x: number) {} }
}
a.b.f(foo);"#;
    let mut s = Session::new_for_test("codeFixAddMissingFunctionDeclaration20", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixMissingFunctionDeclaration")
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_add_missing_param15() {
    let content = r#"function f(a: number, b: number) {}
f();"#;
    let mut s = Session::new_for_test("codeFixAddMissingParam15", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "addMissingParam")
}

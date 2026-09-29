use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_implicit_this_ts_cant_fix_non_function() {
    let content = r#"// @noImplicitThis: true
this;"#;
    let _s = Session::new_for_test("codeFixImplicitThis_ts_cantFixNonFunction", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}

use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_add_missing_const_to_array_destructuring3() {
    let content = r#"let x: any;
[x, y] = [0, 1];"#;
    let _s = Session::new_for_test("codeFixAddMissingConstToArrayDestructuring3", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}

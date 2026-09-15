use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_add_missing_enum_member13() {
    let content = r#"enum E { A, B }
declare var a: E;
a.C;"#;
    let _s = Session::new_for_test("codeFixAddMissingEnumMember13", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixMissingMember")
}

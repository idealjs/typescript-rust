use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_add_missing_enum_member13() {
    let content = r#"enum E { A, B }
declare var a: E;
a.C;"#;
    let mut s = Session::new_for_test("codeFixAddMissingEnumMember13", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "fixMissingMember")
}

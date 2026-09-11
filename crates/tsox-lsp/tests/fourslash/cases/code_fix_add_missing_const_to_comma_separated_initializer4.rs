use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_add_missing_const_to_comma_separated_initializer4() {
    let content = r#"let y: any;
x = 0, y = 0;"#;
    let mut s = Session::new_for_test("codeFixAddMissingConstToCommaSeparatedInitializer4", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}

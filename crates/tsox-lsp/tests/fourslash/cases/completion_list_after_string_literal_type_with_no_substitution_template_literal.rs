use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_string_literal_type_with_no_substitution_template_literal() {
    let content = r#"let count: 'one' | 'two';
count = `[|/**/|]`"#;
    let mut s = Session::new_for_test("completionListAfterStringLiteralTypeWithNoSubstitutionTemplateLiteral", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

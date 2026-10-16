use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_new_identifier_variable_declaration() {
    let content = r#"var y : (s:string, list/*2*/"#;
    let mut s = Session::new_for_test("completionListNewIdentifierVariableDeclaration", content);
    fourslash::verify_completions_empty_at(&mut s, Some("2"));
}

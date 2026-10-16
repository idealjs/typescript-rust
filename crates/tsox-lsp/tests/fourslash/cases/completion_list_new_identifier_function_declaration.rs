use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_new_identifier_function_declaration() {
    let content = r#"// @noLib: true
function F(pref: (a/*1*/"#;
    let mut s = Session::new_for_test("completionListNewIdentifierFunctionDeclaration", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

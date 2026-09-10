use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_new_identifier_function_declaration() {
    let content = r#"// @noLib: true
function F(pref: (a/*1*/"#;
    let mut s = Session::new_for_test("completionListNewIdentifierFunctionDeclaration", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

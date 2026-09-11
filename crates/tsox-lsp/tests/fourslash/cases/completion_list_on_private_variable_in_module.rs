use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_on_private_variable_in_module() {
    let content = r#"namespace Foo {     var testing = "";     test/**/ }"#;
    let mut s = Session::new_for_test("completionListOnPrivateVariableInModule", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_of_module_after_invalid_charater() {
    let content = r#"namespace testModule {
    export var foo = 1;
}
@
testModule./**/"#;
    let mut s = Session::new_for_test("memberListOfModuleAfterInvalidCharater", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

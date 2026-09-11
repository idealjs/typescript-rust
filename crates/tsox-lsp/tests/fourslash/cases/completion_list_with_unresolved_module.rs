use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_with_unresolved_module() {
    let content = r#"namespace m {
    import foo = module('_foo');
    var n: num/**/
}"#;
    let mut s = Session::new_for_test("completionListWithUnresolvedModule", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

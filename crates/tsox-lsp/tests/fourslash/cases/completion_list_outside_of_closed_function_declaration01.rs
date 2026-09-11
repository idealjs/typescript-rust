use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_outside_of_closed_function_declaration01() {
    let content = r#"// no a or b
/*1*/function f (a, b) {}"#;
    let mut s = Session::new_for_test("completionListOutsideOfClosedFunctionDeclaration01", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

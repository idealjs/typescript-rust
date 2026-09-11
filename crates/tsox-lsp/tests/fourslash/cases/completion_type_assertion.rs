use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_type_assertion() {
    let content = r#"// @lib: es5
var x = 'something'
var y = this as/*1*/"#;
    let mut s = Session::new_for_test("completionTypeAssertion", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

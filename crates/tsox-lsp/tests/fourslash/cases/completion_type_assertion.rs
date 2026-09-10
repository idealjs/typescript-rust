use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_type_assertion() {
    let content = r#"// @lib: es5
var x = 'something'
var y = this as/*1*/"#;
    let mut s = Session::new_for_test("completionTypeAssertion", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

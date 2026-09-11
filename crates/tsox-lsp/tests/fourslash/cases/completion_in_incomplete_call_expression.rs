use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_incomplete_call_expression() {
    let content = r#"// @lib: es5
var array = [1, 2, 4]
function a4(x, y, z) { }
a4(...<crash>/**/"#;
    let mut s = Session::new_for_test("completionInIncompleteCallExpression", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

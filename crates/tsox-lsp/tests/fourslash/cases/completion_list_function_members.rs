use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_function_members() {
    let content = r#"// @lib: es5
function fnc1() {
    var bar = 1;
    function foob(){ }
}

fnc1./**/"#;
    let mut s = Session::new_for_test("completionListFunctionMembers", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

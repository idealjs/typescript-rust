use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_inside_target_typed_function() {
    let content = r#"namespace Fix2 {
    interface iFace { (event: string); }
    var foo: iFace = function (elem) { /**/ }
}"#;
    let mut s = Session::new_for_test("completionListInsideTargetTypedFunction", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

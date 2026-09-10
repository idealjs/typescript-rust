use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_inside_target_typed_function() {
    let content = r#"namespace Fix2 {
    interface iFace { (event: string); }
    var foo: iFace = function (elem) { /**/ }
}"#;
    let mut s = Session::new_for_test("completionListInsideTargetTypedFunction", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

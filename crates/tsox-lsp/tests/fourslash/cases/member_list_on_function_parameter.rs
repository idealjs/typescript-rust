use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn member_list_on_function_parameter() {
    let content = r#"namespace Test10 {
    var x: string[] = [];
    x.forEach(function (y) { y./**/} );
}"#;
    let mut s = Session::new_for_test("memberListOnFunctionParameter", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn member_completion_from_function_call() {
    let content = r#"declare interface ifoo {
    text: (value: any) => ifoo;
}
declare var foo: ifoo;
foo.text(function() { })/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ".");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}

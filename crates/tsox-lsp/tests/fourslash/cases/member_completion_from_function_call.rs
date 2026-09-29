use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_completion_from_function_call() {
    let content = r#"declare interface ifoo {
    text: (value: any) => ifoo;
}
declare var foo: ifoo;
foo.text(function() { })/**/"#;
    let mut s = Session::new_for_test("memberCompletionFromFunctionCall", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ".");
    fourslash::verify_completions_exact_at(&mut s, None, &["text"]);
}

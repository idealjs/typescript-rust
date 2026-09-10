use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_after_less_than_token() {
    let content = r#"function f() {
	const k: Record</**/
}"#;
    let mut s = Session::new_for_test("completionsAfterLessThanToken", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}

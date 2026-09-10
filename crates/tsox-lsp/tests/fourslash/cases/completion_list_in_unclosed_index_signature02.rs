use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_unclosed_index_signature02() {
    let content = r#"class C {
    [foo: /*1*/
}"#;
    let mut s = Session::new_for_test("completionListInUnclosedIndexSignature02", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "typeof ");
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["C", "foo"], &[]);
}

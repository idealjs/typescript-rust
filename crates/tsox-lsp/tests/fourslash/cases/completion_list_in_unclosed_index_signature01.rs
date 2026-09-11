use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_index_signature01() {
    let content = r#"class C {
    [foo: string]: typeof /*1*/
}"#;
    let mut s = Session::new_for_test("completionListInUnclosedIndexSignature01", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["foo", "C"], &[]);
}

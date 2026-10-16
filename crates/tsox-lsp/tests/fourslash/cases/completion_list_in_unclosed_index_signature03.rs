use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_index_signature03() {
    let content = r#"class C {
    [foo: string]: { x: typeof /*1*/
}"#;
    let mut s = Session::new_for_test("completionListInUnclosedIndexSignature03", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["foo", "C"], &[]);
    // TODO: }
}

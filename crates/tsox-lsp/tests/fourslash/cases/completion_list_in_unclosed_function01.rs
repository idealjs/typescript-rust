use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_function01() {
    let content = r#"function foo(x: string, y: number, z: boolean) {
    /*1*/
"#;
    let mut s = Session::new_for_test("completionListInUnclosedFunction01", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["foo", "x", "y", "z"], &[]);
    // TODO: }
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_function06() {
    let content = r#"function foo(x: string, y: number, z: boolean) {
    function bar(a: number, b: string = /*1*/, c: typeof x = "hello"
"#;
    let mut s = Session::new_for_test("completionListInUnclosedFunction06", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["foo", "x", "y", "z", "bar", "a"], &[]);
    // TODO: }
}

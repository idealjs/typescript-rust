use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_closed_function03() {
    let content = r#"function foo(x: string, y: number, z: boolean) {
    function bar(a: number, b: string, c: typeof x = /*1*/) {

    }
}"#;
    let mut s = Session::new_for_test("completionListInClosedFunction03", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["foo", "x", "y", "z", "bar", "a", "b"], &[]);
}

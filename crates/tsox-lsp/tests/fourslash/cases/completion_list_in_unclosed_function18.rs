use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn completion_list_in_unclosed_function18() {
    let content = r#"interface MyType {
}

function foo(x: string, y: number, z: boolean) {
    function bar(a: number, b: string = "hello", c: typeof x = "hello") {
        var v = (p: MyType) => y + /*1*/
}"#;
    let mut s = Session::new_for_test("completionListInUnclosedFunction18", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["foo", "x", "y", "z", "bar", "a", "b", "c", "v", "p"], &[]);
    // TODO: }
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_function13() {
    let content = r#"interface MyType {
}

function foo(x: string, y: number, z: boolean) {
    function bar(a: number, b: string = "hello", c: typeof x = "hello") {
        var v = (p: /*1*/
    }
}"#;
    let mut s = Session::new_for_test("completionListInUnclosedFunction13", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["MyType"], &[]);
}

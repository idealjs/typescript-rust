use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_closed_function06() {
    let content = r#"interface MyType {
}

function foo(x: string, y: number, z: boolean) {
    function bar(a: number, b: string = "hello", c: typeof x = "hello") {
        var v = (x: /*1*/);
    }
}"#;
    let mut s = Session::new_for_test("completionListInClosedFunction06", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

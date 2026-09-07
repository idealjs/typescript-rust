use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_on_param() {
    let content = r#"namespace Bar {
    export class Blah { }
}

class Point {
    public Foo(x: Bar./**/Blah, y: Bar.Blah) { }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn completion_list_in_typed_object_literals2() {
    let content = r#"interface Foo {
    x: { a: number };
}
var aaa: Foo;
aaa = { /*9*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "9", &fourslash.CompletionsExpectedList{
    // TODO: }
}

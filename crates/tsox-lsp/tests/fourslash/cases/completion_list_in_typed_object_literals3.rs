use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn completion_list_in_typed_object_literals3() {
    let content = r#"interface Foo {
    x: { a: number };
}
var aaa: Foo;
aaa.x = { /*10*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "10", &fourslash.CompletionsExpectedList{
    // TODO: }
}

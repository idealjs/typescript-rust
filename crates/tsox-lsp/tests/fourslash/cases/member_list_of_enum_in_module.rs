use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn member_list_of_enum_in_module() {
    let content = r#"namespace Fixes {
    enum Foo {
        bar,
        baz
    }
    var f: Foo = Foo./**/;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

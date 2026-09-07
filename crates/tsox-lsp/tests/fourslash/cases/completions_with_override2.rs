use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_with_override2() {
    let content = r#"interface I {
    baz () {}
}
class A {
    foo () {} 
    bar () {}
}
class B extends A implements I {
    override /*1*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

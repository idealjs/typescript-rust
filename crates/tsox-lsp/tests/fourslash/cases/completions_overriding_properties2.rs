use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_properties2() {
    let content = r#"interface I {
    prop: string;
}
class C implements I {
    public pr/**/: string = 'foo';
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

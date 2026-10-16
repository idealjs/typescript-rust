use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_properties3() {
    let content = r#"interface I {
    prop: string;
}
class C implements I {
    public pr/**/: string | number;
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

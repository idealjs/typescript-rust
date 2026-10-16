use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionsWithOverride2", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

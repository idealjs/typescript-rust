use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_object_literal_method5() {
    let content = r#"// @newline: LF
// @Filename: a.ts
interface Foo {
    method(x?: string): void;
}
const foo: Foo = {
    /*m*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "m");
    // TODO: f.VerifyCompletions(t, "m", &fourslash.CompletionsExpectedList{
}

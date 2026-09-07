use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_for_string_literal_quote_preference1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"enum A {
    A,
    B,
    C
}
interface B {
    a: keyof typeof A;
}
const b: B = {
    a: /**/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

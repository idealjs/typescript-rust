use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_for_string_literal_quote_preference() {
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
    let mut s = Session::new_for_test("completionForStringLiteral_quotePreference", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

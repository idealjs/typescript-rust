use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_for_string_literal_quote_preference5() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"type T = "0" | "1";
const t: T = /**/"#;
    let mut s = Session::new_for_test("completionForStringLiteral_quotePreference5", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

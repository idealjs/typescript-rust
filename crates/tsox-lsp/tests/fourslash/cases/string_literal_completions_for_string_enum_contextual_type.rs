use tsox_lsp::fourslash::{self, Session};


#[test]
fn string_literal_completions_for_string_enum_contextual_type() {
    let content = r#"const enum E {
    A = "A",
}
const e: E = "/**/";"#;
    let mut s = Session::new_for_test("stringLiteralCompletionsForStringEnumContextualType", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

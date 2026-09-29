use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_dot_in_array_literal_in_object_literal() {
    let content = r#"const o = { x: [[|.|][||]/**/"#;
    let mut s = Session::new_for_test("completionsDotInArrayLiteralInObjectLiteral", content);
    // TODO: f.VerifyNonSuggestionDiagnostics(t, []*lsproto.Diagnostic{
    fourslash::verify_completions_empty_at(&mut s, Some(""));
    // TODO: }
}

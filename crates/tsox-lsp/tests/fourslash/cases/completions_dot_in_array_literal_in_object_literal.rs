use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn completions_dot_in_array_literal_in_object_literal() {
    let content = r#"const o = { x: [[|.|][||]/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, []*lsproto.Diagnostic{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", nil)
    // TODO: }
}

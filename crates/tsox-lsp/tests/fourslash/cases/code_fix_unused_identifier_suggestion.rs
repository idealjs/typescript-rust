use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_unused_identifier_suggestion() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
function f([|p|]) {
    const [|x|] = 0;
}"#;
    let mut s = Session::new_for_test("codeFixUnusedIdentifier_suggestion", content);
    // TODO: f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
    // TODO: f.VerifyCodeFixAvailable(t, nil)
}

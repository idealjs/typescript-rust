use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_unused_identifier_suggestion() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
function f([|p|]) {
    const [|x|] = 0;
}"#;
    let mut s = Session::new_for_test("codeFixUnusedIdentifier_suggestion", content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, nil)
}

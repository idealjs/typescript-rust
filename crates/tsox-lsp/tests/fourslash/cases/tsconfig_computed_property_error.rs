use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn tsconfig_computed_property_error() {
    let content = r#"// @filename: tsconfig.json
{
    [|["oops!" + 42]|]: "true",
    "compilerOptions": { "lib": ["es5"] },
    "files": [
        "nonexistentfile.ts"
    ],
    "compileOnSave": true
}"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, []*lsproto.Diagnostic{
}

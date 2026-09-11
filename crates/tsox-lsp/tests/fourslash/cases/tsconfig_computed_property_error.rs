use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("tsconfigComputedPropertyError", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyNonSuggestionDiagnostics(t, []*lsproto.Diagnostic{
}

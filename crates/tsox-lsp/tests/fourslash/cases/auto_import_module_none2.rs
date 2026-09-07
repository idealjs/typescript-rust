use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_module_none2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @module: none
// @moduleResolution: bundler
// @target: es2015
// @Filename: /node_modules/dep/index.d.ts
export const x: number;
// @Filename: /index.ts
 x/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { x } from 'dep'; x;")
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, nil)
}

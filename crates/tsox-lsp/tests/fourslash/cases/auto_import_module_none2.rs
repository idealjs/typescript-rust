use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("autoImportModuleNone2", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.ReplaceLine(t, 0, "import { x } from 'dep'; x;")
    // TODO: f.VerifyNonSuggestionDiagnostics(t, nil)
}

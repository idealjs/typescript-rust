use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_import_module_specifier_ending_js() {
    let content = r#"//@allowJs: true
//@Filename:test.js
export function f(){
    return 1
}
//@Filename:module.js
import { f } from ".//**/""#;
    let mut s = Session::new_for_test("completionImportModuleSpecifierEndingJs", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

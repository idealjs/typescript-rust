use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_import_module_specifier_ending_ts() {
    let content = r#"//@Filename:test.ts
export function f(){
    return 1
}
//@Filename:module.ts
import { f } from ".//**/""#;
    let mut s = Session::new_for_test("completionImportModuleSpecifierEndingTs", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

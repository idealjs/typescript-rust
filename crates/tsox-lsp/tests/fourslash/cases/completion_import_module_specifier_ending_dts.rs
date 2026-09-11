use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_import_module_specifier_ending_dts() {
    let content = r#"//@Filename:test.d.ts
 export declare class Test {}
//@Filename:module.ts
import { Test } from ".//**/""#;
    let mut s = Session::new_for_test("completionImportModuleSpecifierEndingDts", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

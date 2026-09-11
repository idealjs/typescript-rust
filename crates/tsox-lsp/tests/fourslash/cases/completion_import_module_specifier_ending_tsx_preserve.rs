use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_import_module_specifier_ending_tsx_preserve() {
    let content = r#"//@jsx:preserve
//@Filename:test.tsx
 export class Test { }
//@Filename:module.tsx
import { Test } from ".//**/""#;
    let mut s = Session::new_for_test("completionImportModuleSpecifierEndingTsxPreserve", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

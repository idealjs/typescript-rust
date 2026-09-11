use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_import_module_specifier_ending_jsx() {
    let content = r#"//@allowJs: true
//@jsx:preserve
//@Filename:test.jsx
 export class Test { }
//@Filename:module.jsx
import { Test } from ".//**/""#;
    let mut s = Session::new_for_test("completionImportModuleSpecifierEndingJsx", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

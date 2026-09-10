use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

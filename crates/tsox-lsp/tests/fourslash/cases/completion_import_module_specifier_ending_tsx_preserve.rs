use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_import_module_specifier_ending_tsx_preserve() {
    let content = r#"//@jsx:preserve
//@Filename:test.tsx
 export class Test { }
//@Filename:module.tsx
import { Test } from ".//**/""#;
    let mut s = Session::new_for_test("completionImportModuleSpecifierEndingTsxPreserve", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

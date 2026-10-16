use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_import_module_specifier_ending_tsx_react() {
    let content = r#"//@jsx:react
//@Filename:test.tsx
 export class Test { }
//@Filename:module.tsx
import { Test } from ".//**/""#;
    let mut s = Session::new_for_test("completionImportModuleSpecifierEndingTsxReact", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

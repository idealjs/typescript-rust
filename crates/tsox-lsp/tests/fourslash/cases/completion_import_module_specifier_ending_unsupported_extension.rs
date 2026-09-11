use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_import_module_specifier_ending_unsupported_extension() {
    let content = r#"//@Filename:index.css
 body {}
//@Filename:module.ts
import ".//**/""#;
    let mut s = Session::new_for_test("completionImportModuleSpecifierEndingUnsupportedExtension", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_import_module_specifier_ending_unsupported_extension() {
    let content = r#"//@Filename:index.css
 body {}
//@Filename:module.ts
import ".//**/""#;
    let mut s = Session::new_for_test("completionImportModuleSpecifierEndingUnsupportedExtension", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

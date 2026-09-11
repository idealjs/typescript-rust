use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_after_import_with_js_doc() {
    let content = r#"// @Filename: /index.ts
/** hello! */
import /**/"#;
    let mut s = Session::new_for_test("completionAfterImportWithJSDoc", content);
    // TODO: // Should not crash when requesting completions after import preceded by JSDoc
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

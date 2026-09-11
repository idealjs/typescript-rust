use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_default_keyword_when_default_export_available() {
    let content = r#"// @filename: index.ts
export default function () {}
def/*1*/"#;
    let mut s = Session::new_for_test("completionsDefaultKeywordWhenDefaultExportAvailable", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

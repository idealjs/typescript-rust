use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_details_with_misspelled_name() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /a.ts
export const abc = 0;
// @Filename: /b.ts
acb/*1*/;
// @Filename: /c.ts
acb/*2*/;"#;
    let mut s = Session::new_for_test("completionsImport_details_withMisspelledName", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("2"), &fourslash.ApplyCodeActionFromCompletionOptions{
}

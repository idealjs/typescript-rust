use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_quote_style() {
    let content = r#"// @module: esnext
// @Filename: /a.ts
export const foo = 0;
// @Filename: /b.ts
fo/**/"#;
    let mut s = Session::new_for_test("completionsImport_quoteStyle", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}

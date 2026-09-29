use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_quote_detection() {
    let content = r#"// @module: esnext
// @Filename: /a.ts
export const foo = 0;
// @Filename: /b.ts
import {} from 'node:path';

fo/**/"#;
    let mut s = Session::new_for_test("autoImportQuoteDetection", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}

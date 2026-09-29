use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_fixes_quote_preference_double_import_helpers() {
    let content = r#"// @importHelpers: true
// @filename: /a.ts
export default () => {};
// @filename: /b.ts
export default () => {};
// @filename: /test.ts
import a from "./a";
[|b|];"#;
    let mut s = Session::new_for_test("importFixes_quotePreferenceDouble_importHelpers", content);
    fourslash::go_to_file(&mut s, "/test.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

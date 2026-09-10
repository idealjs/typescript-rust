use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
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
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

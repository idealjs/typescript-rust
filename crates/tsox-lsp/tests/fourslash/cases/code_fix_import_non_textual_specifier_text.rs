use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_import_non_textual_specifier_text() {
    let content = r#"// @Filename: /a.ts
import type { A } from `./${myFolder}/${myFile}`;

new A/**/()"#;
    let mut s = Session::new_for_test("codeFixImportNonTextualSpecifierText", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

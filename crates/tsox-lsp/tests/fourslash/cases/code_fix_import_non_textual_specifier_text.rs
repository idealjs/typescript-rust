use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: '\n' +"]
#[test]
fn code_fix_import_non_textual_specifier_text() {
    // TODO: const content = "// @Filename: /a.ts\n" +
    // TODO: "import type { A } from `./${myFolder}/${myFile}`;\n" +
    // TODO: "\n" +
    // TODO: "new A/**/()"
    let mut s = Session::new_for_test("codeFixImportNonTextualSpecifierText", "");
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_import_type() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: /a.js
export {};
/** @typedef {number} T */
// @Filename: /b.js
/** @type {T} */
const x = 0;"#;
    let mut s = Session::new_for_test("importNameCodeFix_importType", content);
    fourslash::go_to_file(&mut s, "/b.js");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

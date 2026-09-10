use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_file_quote_style2() {
    let content = r#"[|import m2 = require('./module2');

f1/*0*/();|]
// @Filename: module1.ts
export function f1() {}
// @Filename: module2.ts
export var v2 = 6;"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportFileQuoteStyle2", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

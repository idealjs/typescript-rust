use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_file_quote_style_mixed1() {
    let content = r#"[|import { v2 } from './module2';
import { v3 } from "./module3";

f1/*0*/();|]
// @Filename: module1.ts
export function f1() {}
// @Filename: module2.ts
export var v2 = 6;
// @Filename: module3.ts
export var v3 = 6;"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportFileQuoteStyleMixed1", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

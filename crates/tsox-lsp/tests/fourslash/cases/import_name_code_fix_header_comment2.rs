use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_header_comment2() {
    let content = r#"// @Filename: /a.ts
export const foo = 0;
// @Filename: /b.ts
export const bar = 0;
// @Filename: /c.ts
/*--------------------
 *  Copyright Header
 *--------------------*/

const afterHeader = 1;

// non-header comment
import { bar } from "./b";
foo;"#;
    let mut s = Session::new_for_test("importNameCodeFix_HeaderComment2", content);
    fourslash::go_to_file(&mut s, "/c.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

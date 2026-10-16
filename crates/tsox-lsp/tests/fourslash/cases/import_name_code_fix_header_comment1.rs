use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_header_comment1() {
    let content = r#"// @Filename: /a.ts
export const foo = 0;
// @Filename: /b.ts
export const bar = 0;
// @Filename: /c.ts
/*--------------------
 *  Copyright Header
 *--------------------*/

import { bar } from "./b";
foo;"#;
    let mut s = Session::new_for_test("importNameCodeFix_HeaderComment1", content);
    fourslash::go_to_file(&mut s, "/c.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

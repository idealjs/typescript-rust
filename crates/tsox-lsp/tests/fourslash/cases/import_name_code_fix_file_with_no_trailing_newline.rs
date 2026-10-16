use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_file_with_no_trailing_newline() {
    let content = r#"// @Filename: /a.ts
export const foo = 0;
// @Filename: /b.ts
export const bar = 0;
// @Filename: /c.ts
foo;
import { bar } from "./b";"#;
    let mut s = Session::new_for_test("importNameCodeFix_fileWithNoTrailingNewline", content);
    fourslash::go_to_file(&mut s, "/c.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_with_json() {
    let content = r#"// @Filename: /a.ts
export const a = 'a';
// @Filename: /b.ts
import "./anything.json";

a/**/"#;
    let mut s = Session::new_for_test("importNameCodeFix_withJson", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_js_rename() {
    let content = r#"
// @Filename: /tsconfig.json
{ "compilerOptions": { "module": "nodenext" } }
// @Filename: /a.ts
export const a = 1;
// @Filename: /b.ts
import { a } from ".//*rename*/a.js";"#;
    let _s = Session::new_for_test("getEditsForFileRename_jsRename", content);
    // TODO: f.VerifyRename(t, "rename", "c.js", map[string]string{
}

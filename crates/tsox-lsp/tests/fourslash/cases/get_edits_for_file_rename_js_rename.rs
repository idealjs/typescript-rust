use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyRename"]
#[test]
fn get_edits_for_file_rename_js_rename() {
    let content = r#"
// @Filename: /tsconfig.json
{ "compilerOptions": { "module": "nodenext" } }
// @Filename: /a.ts
export const a = 1;
// @Filename: /b.ts
import { a } from ".//*rename*/a.js";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRename"); // f.VerifyRename(t, "rename", "c.js", map[string]string{
}

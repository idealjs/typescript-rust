use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_js_extension() {
    let content = r#"// @allowJs: true
// @Filename: /src/a.js
export const a = 0;
// @Filename: /b.js
import { a } from "./src/a.js";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/b.js", "/src/b.js", map[string]string{
}

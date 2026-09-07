use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_js_simple() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
import b from "./b.js";
// @Filename: /b.js
module.exports = 1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/b.js", "/c.js", map[string]string{
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_preserve_path_ending() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @strict: true
// @jsx: preserve
// @resolveJsonModule: true
// @Filename: /index.js
export const x = 0;
// @Filename: /jsx.jsx
export const y = 0;
// @Filename: /j.jonah.json
{ "j": 0 }
// @Filename: /a.js
import { x as x0 } from ".";
import { x as x1 } from "./index";
import { x as x2 } from "./index.js";
import { y } from "./jsx.jsx";
import { j } from "./j.jonah.json";"#;
    let mut s = Session::new(content);
    fourslash::verify_no_errors(&mut s);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/a.js", "/b.js", map[string]string{}, nil /*preferences*/)
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/b.js", "/src/b.js", map[string]string{
}

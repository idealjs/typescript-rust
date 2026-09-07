use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_case_insensitive() {
    let content = r#"// @useCaseSensitiveFileNames: false
// @Filename: /a.ts
export const a = 0;
// @Filename: /b.ts
import { a } from "./A";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/a.ts", "/eh.ts", map[string]string{
}

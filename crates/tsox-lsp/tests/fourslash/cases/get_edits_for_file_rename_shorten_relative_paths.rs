use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_shorten_relative_paths() {
    let content = r#"// @Filename: /src/foo/x.ts

// @Filename: /src/old.ts
import { x } from "./foo/x";"#;
    let mut s = Session::new_for_test("getEditsForFileRename_shortenRelativePaths", content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/src/old.ts", "/src/foo/new.ts", map[string]string{
}

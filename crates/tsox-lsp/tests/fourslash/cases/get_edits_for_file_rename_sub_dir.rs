use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_sub_dir() {
    let content = r#"// @Filename: /src/foo/a.ts

// @Filename: /src/old.ts
import a from "./foo/a";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/src/old.ts", "/src/dir/new.ts", map[string]string{
}

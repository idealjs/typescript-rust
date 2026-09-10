use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_symlink() {
    let content = r#"// @Filename: /foo.ts
// @Symlink: /node_modules/foo/index.ts
export const x = 0;
// @Filename: /user.ts
import { x } from 'foo';"#;
    let mut s = Session::new_for_test("getEditsForFileRename_symlink", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/user.ts", "/luser.ts", map[string]string{}, nil /*preferences*/)
}

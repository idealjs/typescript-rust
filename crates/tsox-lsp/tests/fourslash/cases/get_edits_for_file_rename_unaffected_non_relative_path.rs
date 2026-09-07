use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_unaffected_non_relative_path() {
    let content = r#"// @Filename: /sub/a.ts
export const a = 1;
// @Filename: /sub/b.ts
import { a } from "sub/a";
// @Filename: /tsconfig.json
{"compilerOptions":{"paths":{"*":["*"]}}}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/sub/b.ts", "/sub/c/d.ts", map[string]string{}, nil /*preferences*/
}

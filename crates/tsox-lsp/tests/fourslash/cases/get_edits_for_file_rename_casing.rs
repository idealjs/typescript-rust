use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_casing() {
    let content = r#"// @Filename: /a.ts
import { foo } from "./dir/fOo";
// @Filename: /dir/fOo.ts
export const foo = 0;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/dir", "/newDir", map[string]string{
}

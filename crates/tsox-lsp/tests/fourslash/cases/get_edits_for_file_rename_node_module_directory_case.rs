use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_node_module_directory_case() {
    let content = r#"// @Filename: /a/b/file1.ts
import { foo } from "foo";
// @Filename: /a/node_modules/foo/index.d.ts
export const foo = 0;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/a/b", "/a/B", map[string]string{}, nil /*preferences*/)
}

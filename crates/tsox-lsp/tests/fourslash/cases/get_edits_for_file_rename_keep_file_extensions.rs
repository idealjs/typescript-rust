use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_keep_file_extensions() {
    let content = r#"// @Filename: /tsconfig.json
{
  "compilerOptions": {
    "module": "Node16",
    "rootDirs": ["src"]
  }
}
// @Filename: /src/person.ts
export const name = 0;
// @Filename: /src/index.ts
import {name} from "./person.js";"#;
    let mut s = Session::new_for_test("getEditsForFileRename_keepFileExtensions", content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/src/person.ts", "/src/vip.ts", map[string]string{
}

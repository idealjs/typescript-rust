use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_rename_to_index() {
    let content = r#"// @Filename: /a.ts
/// <reference path="./src/old.ts" />
import old from "./src/old";
// @Filename: /src/a.ts
/// <reference path="./old.ts" />
import old from "./old";
// @Filename: /src/foo/a.ts
/// <reference path="../old.ts" />
import old from "../old";
// @Filename: /src/old.ts

// @Filename: /tsconfig.json
{ "files": ["a.ts", "src/a.ts", "src/foo/a.ts", "src/old.ts"] }"#;
    let _s = Session::new_for_test("getEditsForFileRename_renameToIndex", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/src/old.ts", "/src/index.ts", map[string]string{
}

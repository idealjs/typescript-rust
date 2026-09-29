use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_rename_from_index() {
    let content = r#"// @Filename: /a.ts
/// <reference path="./src/index.ts" />
import old from "./src";
import old2 from "./src/index";
// @Filename: /src/a.ts
/// <reference path="./index.ts" />
import old from ".";
import old2 from "./index";
// @Filename: /src/foo/a.ts
/// <reference path="../index.ts" />
import old from "..";
import old2 from "../index";
// @Filename: /src/index.ts

// @Filename: /tsconfig.json
{ "files": ["a.ts", "src/a.ts", "src/foo/a.ts", "src/index.ts"] }"#;
    let _s = Session::new_for_test("getEditsForFileRename_renameFromIndex", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/src/index.ts", "/src/new.ts", map[string]string{
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_directory_down() {
    let content = r#"// @Filename: /a.ts
/// <reference path="./src/old/file.ts" />
import old from "./src/old";
import old2 from "./src/old/file";
export default 0;
// @Filename: /src/b.ts
/// <reference path="./old/file.ts" />
import old from "./old";
import old2 from "./old/file";
export default 0;
// @Filename: /src/foo/c.ts
/// <reference path="../old/file.ts" />
import old from "../old";
import old2 from "../old/file";
export default 0;
// @Filename: /src/old/index.ts
import a from "../../a";
import a2 from "../b";
import a3 from "../foo/c";
import f from "./file";
export default 0;
// @Filename: /src/old/file.ts
export default 0;
// @Filename: /tsconfig.json
{ "files": ["a.ts", "src/b.ts", "src/foo/c.ts", "src/old/index.ts", "src/old/file.ts"] }"#;
    let mut s = Session::new_for_test("getEditsForFileRename_directory_down", content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/src/old", "/src/newDir/new", map[string]string{
}

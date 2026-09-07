use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename() {
    let content = r#"// @Filename: /a.ts
/// <reference path="./src/old.ts" />
import old from "./src/old";
// @Filename: /src/a.ts
/// <reference path="./old.ts" />
import old from "./old";
// @Filename: /src/foo/a.ts
/// <reference path="../old.ts" />
import old from "../old";
// @Filename: /unrelated.ts
import { x } from "././src/./foo/./a";
// @Filename: /src/old.ts
export default 0;
// @Filename: /tsconfig.json
{ "files": ["a.ts", "src/a.ts", "src/foo/a.ts", "src/old.ts"] }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/src/old.ts", "/src/new.ts", map[string]string{
}

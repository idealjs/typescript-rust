use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // The parent-directory solution tsconfig only references th"]
#[test]
fn get_edits_for_file_rename_with_solution_config_file() {
    // TODO: // The parent-directory solution tsconfig only references the composite child
    // TODO: // project, so when the child file is opened the solution is created as an
    // TODO: // ancestor project without ever building its program (it stays nil). Renaming
    // TODO: // a file in the child project must not crash when iterating that nil-program
    // TODO: // solution project.
    let content = r#"
// @Filename: /tsconfig.json
{
  "files": [],
  "references": [
    { "path": "./src/tsconfig.json" }
  ]
}

// @Filename: /src/tsconfig.json
{
  "compilerOptions": {
    "composite": true
  },
  "files": ["./a.ts", "./b.ts"]
}

// @Filename: /src/a.ts
import { b } from "./b";
b;

// @Filename: /src/b.ts
export const b = 0;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/src/b.ts", "/src/c.ts", map[string]string{
}

#[ignore = "generator: for _, change := range *result.WorkspaceEdit.DocumentChanges"]
#[test]
fn get_edits_for_file_rename_loads_unopened_composite_project() {
    let content = r#"
// @stateBaseline: true
// @Filename: /tsconfig.json
{
  "files": [],
  "references": [
    { "path": "./lib" },
    { "path": "./app" }
  ]
}

// @Filename: /lib/tsconfig.json
{
  "compilerOptions": {
    "composite": true
  },
  "files": ["./helper.ts", "./other-helper.ts"]
}

// @Filename: /lib/helper.ts
export const /*helper*/helper = 0;

// @Filename: /lib/other-helper.ts
import { helper } from "./helper";
helper;

// @Filename: /app/tsconfig.json
{
  "compilerOptions": {
    "composite": true
  },
  "files": ["./main.ts"],
  "references": [
    { "path": "../lib" }
  ]
}

// @Filename: /app/main.ts
import { helper } from "../lib/helper";
helper;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "helper");
    // TODO: result := f.WillRenameFiles(t, &lsproto.FileRename{
    // TODO: if result.WorkspaceEdit == nil || result.WorkspaceEdit.DocumentChanges == nil {
    // TODO: for _, change := range *result.WorkspaceEdit.DocumentChanges {
    // TODO: t.Fatal("workspace/willRenameFiles returned no import update for /app/main.ts")
}

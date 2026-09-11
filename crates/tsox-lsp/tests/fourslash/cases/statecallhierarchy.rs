use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_across_project() {
    let content = r#"
// @stateBaseline: true
// @Filename: /projects/temp/temp.ts
/*temp*/let x = 10
// @Filename: /projects/temp/tsconfig.json
{}
// @Filename: /projects/container/lib/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
	references: [],
	files: [
		"index.ts",
		"bar.ts",
		"baz.ts"
	],
}
// @Filename: /projects/container/lib/index.ts
export function /*call*/createModelReference() {}
// @Filename: /projects/container/lib/bar.ts
import { createModelReference } from "./index";
function openElementsAtEditor() {
  createModelReference();
}
// @Filename: /projects/container/lib/baz.ts
import { createModelReference } from "./index";
function registerDefaultLanguageCommand() {
  createModelReference();
}
// @Filename: /projects/container/exec/tsconfig.json
{
	"files": ["./index.ts"],
	"references": [
		{ "path": "../lib" },
	],
}
// @Filename: /projects/container/exec/index.ts
import { createModelReference } from "../lib";
function openElementsAtEditor1() {
  createModelReference();
}
// @Filename: /projects/container/compositeExec/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
	"files": ["./index.ts"],
	"references": [
		{ "path": "../lib" },
	],
}
// @Filename: /projects/container/compositeExec/index.ts
import { createModelReference } from "../lib";
function openElementsAtEditor2() {
  createModelReference();
}
// @Filename: /projects/container/tsconfig.json
{
	"files": [],
	"include": [],
	"references": [
		{ "path": "./exec" },
		{ "path": "./compositeExec" },
	],
}
// @Filename: /projects/container/tsconfig.json
{
	"files": [],
	"include": [],
	"references": [
		{ "path": "./exec" },
		{ "path": "./compositeExec" },
	],
}
// @Filename: /projects/container/tsconfig.json
{
	"files": [],
	"include": [],
	"references": [
		{ "path": "./exec" },
		{ "path": "./compositeExec" },
	],
}"#;
    let mut s = Session::new_for_test("callHierarchyAcrossProject", content);
    fourslash::go_to_marker(&mut s, "call");
    // TODO: // Open temp file and verify all projects alive
    fourslash::go_to_marker(&mut s, "temp");
    // TODO: // Ref projects are loaded after as part of this command
    fourslash::go_to_marker(&mut s, "call");
    // TODO: f.VerifyBaselineCallHierarchy(t)
    // TODO: // Open temp file and verify all projects alive
    // TODO: f.CloseFileOfMarker(t, "temp")
    fourslash::go_to_marker(&mut s, "temp");
    // TODO: // Close all files and open temp file, only inferred project should be alive
    // TODO: f.CloseFileOfMarker(t, "call")
    // TODO: f.CloseFileOfMarker(t, "temp")
    fourslash::go_to_marker(&mut s, "temp");
}

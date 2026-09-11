use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_ancestor_project_ref_mangement() {
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
	],
}
// @Filename: /projects/container/lib/index.ts
export const myConst = 30;
// @Filename: /projects/container/exec/tsconfig.json
{
	"files": ["./index.ts"],
	"references": [
		{ "path": "../lib" },
	],
}
// @Filename: /projects/container/exec/index.ts
import { myConst } from "../lib";
export function getMyConst() {
	return myConst;
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
import { /*find*/myConst } from "../lib";
export function getMyConst() {
	return myConst;
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
}
"#;
    let mut s = Session::new_for_test("renameAncestorProjectRefMangement", content);
    fourslash::go_to_marker(&mut s, "find");
    // TODO: // Open temp file and verify all projects alive
    fourslash::go_to_marker(&mut s, "temp");
    // TODO: // Ref projects are loaded after as part of this command
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "find")
    // TODO: // Open temp file and verify all projects alive
    // TODO: f.CloseFileOfMarker(t, "temp")
    fourslash::go_to_marker(&mut s, "temp");
    // TODO: // Close all files and open temp file, only inferred project should be alive
    // TODO: f.CloseFileOfMarker(t, "find")
    // TODO: f.CloseFileOfMarker(t, "temp")
    fourslash::go_to_marker(&mut s, "temp");
}

#[test]
fn rename_in_common_file() {
    let content = r#"
// @stateBaseline: true
// @Filename: /projects/a/a.ts
/*aTs*/import {C} from "./c/fc";
console.log(C)
// @Filename: /projects/a/tsconfig.json
{}
// @link:  /projects/c -> /projects/a/c
// @Filename: /projects/b/b.ts
/*bTs*/import {C} from "../c/fc";
console.log(C)
// @Filename: /projects/b/tsconfig.json
{}
// @link:  /projects/c -> /projects/b/c
// @Filename: /projects/c/fc.ts
export const /*find*/C = 42;
"#;
    let mut s = Session::new_for_test("renameInCommonFile", content);
    fourslash::go_to_marker(&mut s, "aTs");
    fourslash::go_to_marker(&mut s, "bTs");
    // TODO: findMarker := f.MarkerByName(t, "find")
    // TODO: aFcMarker := findMarker.MakerWithSymlink("/projects/a/c/fc.ts")
    // TODO: f.GoToMarkerOrRange(t, aFcMarker)
    // TODO: f.GoToMarkerOrRange(t, findMarker.MakerWithSymlink("/projects/b/c/fc.ts"))
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, aFcMarker)
}

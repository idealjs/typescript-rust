use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_lens_across_projects() {
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
		"bar.ts"
	],
}
// @Filename: /projects/container/lib/index.ts
/*impl*/
export interface Pointable {
  getX(): number;
  getY(): number;
}
export const val = 42;
// @Filename: /projects/container/lib/bar.ts
import { Pointable } from "./index";
class Point implements Pointable {
  getX(): number {
    return 0;
  }
  getY(): number {
    return 0;
  }
}
// @Filename: /projects/container/exec/tsconfig.json
{
	"files": ["./index.ts"],
	"references": [
		{ "path": "../lib" },
	],
}
// @Filename: /projects/container/exec/index.ts
import { Pointable } from "../lib";
class Point1 implements Pointable {
  getX(): number {
    return 0;
  }
  getY(): number {
    return 0;
  }
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
import { Pointable } from "../lib";
class Point2 implements Pointable {
  getX(): number {
    return 0;
  }
  getY(): number {
    return 0;
  }
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
    let mut s = Session::new_for_test("codeLensAcrossProjects", content);
    fourslash::go_to_marker(&mut s, "impl");
    // TODO: // Open temp file and verify all projects alive
    fourslash::go_to_marker(&mut s, "temp");
    // TODO: // Ref projects are loaded after as part of this command
    // TODO: f.VerifyBaselineCodeLens(t, &lsutil.UserPreferences{
    // TODO: // Open temp file and verify all projects alive
    // TODO: f.CloseFileOfMarker(t, "temp")
    fourslash::go_to_marker(&mut s, "temp");
    // TODO: // Close all files and open temp file, only inferred project should be alive
    // TODO: f.CloseFileOfMarker(t, "impl")
    // TODO: f.CloseFileOfMarker(t, "temp")
    fourslash::go_to_marker(&mut s, "temp");
}

#[test]
fn code_lens_on_function_across_projects1() {
    let content = r#"
// @filename: ./a/tsconfig.json
{
  "compilerOptions": {
	"composite": true,
	"declaration": true,
	"declarationMaps": true,
	"outDir": "./dist",
	"rootDir": "src"
  },
  "include": ["./src"]
}

// @filename: ./a/src/foo.ts
export function aaa() {}
aaa();

// @filename: ./b/tsconfig.json
{
  "compilerOptions": {
	"composite": true,
	"declaration": true,
	"declarationMaps": true,
	"outDir": "./dist",
	"rootDir": "src"
  },
  "references": [{ "path": "../a" }],
  "include": ["./src"]
}

// @filename: ./b/src/bar.ts
import * as foo from '../../a/dist/foo.js';
foo.aaa();
"#;
    let _s = Session::new_for_test("codeLensOnFunctionAcrossProjects1", content);
    // TODO: f.VerifyBaselineCodeLens(t, &lsutil.UserPreferences{
}

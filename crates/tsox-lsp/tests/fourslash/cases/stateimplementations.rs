use tsox_lsp::fourslash::{self, Session};


#[test]
fn implementations_across_projects() {
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
export interface /*impl*/Foo {
    func();
}
export const val = 42;
// @Filename: /projects/container/lib/bar.ts
import {Foo} from './index'
class A implements Foo {
    func() {}
}
class B implements Foo {
    func() {}
}
// @Filename: /projects/container/exec/tsconfig.json
{
	"files": ["./index.ts"],
	"references": [
		{ "path": "../lib" },
	],
}
// @Filename: /projects/container/exec/index.ts
import { Foo } from "../lib";
class A1 implements Foo {
    func() {}
}
class B1 implements Foo {
    func() {}
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
import { Foo } from "../lib";
class A2 implements Foo {
    func() {}
}
class B2 implements Foo {
    func() {}
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
    let mut s = Session::new_for_test("implementationsAcrossProjects", content);
    fourslash::go_to_marker(&mut s, "impl");
    // TODO: // Open temp file and verify all projects alive
    fourslash::go_to_marker(&mut s, "temp");
    // TODO: // Ref projects are loaded after as part of this command
    // TODO: f.VerifyBaselineGoToImplementation(t, "impl")
    // TODO: // Open temp file and verify all projects alive
    // TODO: f.CloseFileOfMarker(t, "temp")
    fourslash::go_to_marker(&mut s, "temp");
    // TODO: // Close all files and open temp file, only inferred project should be alive
    // TODO: f.CloseFileOfMarker(t, "impl")
    // TODO: f.CloseFileOfMarker(t, "temp")
    fourslash::go_to_marker(&mut s, "temp");
}

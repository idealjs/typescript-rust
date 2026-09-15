use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_solution_referencing_default_project_directly() {
    let content = r#"
// @stateBaseline: true 
// @tsc: --build /myproject/tsconfig.json
// @Filename: dummy/dummy.ts
/*dummy*/const x = 1;
// @Filename: dummy/tsconfig.json
{ }
// @Filename: myproject/tsconfig.json
{
	"files": [],
	"references": [{ "path": "./tsconfig-src.json" }]
}
// @Filename: myproject/tsconfig-src.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target",
		"declarationMap": true,
	},
	"include": ["./src/\**/*"]
}
// @Filename: myproject/src/main.ts
import { foo } from './helpers/functions';
export { /*mainFoo*/foo };
// @Filename: myproject/src/helpers/functions.ts
export function foo() { return 1; }
// @Filename: myproject/indirect3/tsconfig.json
{ }
// @Filename: myproject/indirect3/main.ts
import { /*fooIndirect3Import*/foo } from '../target/src/main';
foo()
export function bar() {}
"#;
    let mut s = Session::new_for_test("findAllRefsSolutionReferencingDefaultProjectDirectly", content);
    // TODO: // Ensure configured project is found for open file
    fourslash::go_to_marker(&mut s, "mainFoo");
    // TODO: // !!! TODO Verify errors
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: // Projects lifetime
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: // Find all refs in default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "mainFoo")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    // TODO: // Find all ref in non default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "fooIndirect3Import")
}

#[test]
fn find_all_refs_solution_referencing_default_project_indirectly() {
    let content = r#"
// @stateBaseline: true 
// @tsc: --build /myproject/tsconfig.json
// @Filename: dummy/dummy.ts
/*dummy*/const x = 1;
// @Filename: dummy/tsconfig.json
{ }
// @Filename: myproject/tsconfig.json
{
	"files": [],
	"references":  [
		{ "path": "./tsconfig-indirect1.json" },
		{ "path": "./tsconfig-indirect2.json" },
	]
}
// @Filename: myproject/tsconfig-src.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target",
		"declarationMap": true,
	},
	"include": ["./src/\**/*"]
}
// @Filename: myproject/src/main.ts
import { foo } from './helpers/functions';
export { /*mainFoo*/foo };
// @Filename: myproject/src/helpers/functions.ts
export function foo() { return 1; }
// @Filename: myproject/indirect3/tsconfig.json
{ }
// @Filename: myproject/indirect3/main.ts
import { /*fooIndirect3Import*/foo } from '../target/src/main';
foo()
export function bar() {}
// @FileName: myproject/indirect1/main.ts
export const indirect = 1;
// @Filename: myproject/tsconfig-indirect1.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target/",
	},
	"files": [
		"./indirect1/main.ts"
	],
	"references": [
		{
			"path": "./tsconfig-src.json"
		}
	]
}
// @FileName: myproject/indirect2/main.ts
export const indirect = 1;
// @Filename: myproject/tsconfig-indirect2.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target/",
	},
	"files": [
		"./indirect2/main.ts"
	],
	"references": [
		{
			"path": "./tsconfig-src.json"
		}
	]
}
"#;
    let mut s = Session::new_for_test("findAllRefsSolutionReferencingDefaultProjectIndirectly", content);
    // TODO: // Ensure configured project is found for open file
    fourslash::go_to_marker(&mut s, "mainFoo");
    // TODO: // !!! TODO Verify errors
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: // Projects lifetime
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: // Find all refs in default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "mainFoo")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    // TODO: // Find all ref in non default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "fooIndirect3Import")
}

#[test]
fn find_all_refs_solution_with_disable_referenced_project_load_referencing_default_project_directly() {
    let content = r#"
// @stateBaseline: true 
// @tsc: --build /myproject/tsconfig.json
// @Filename: dummy/dummy.ts
/*dummy*/const x = 1;
// @Filename: dummy/tsconfig.json
{ }
// @Filename: myproject/tsconfig.json
{
	"compilerOptions": {
		"disableReferencedProjectLoad": true
	},
	"files": [],
	"references": [{ "path": "./tsconfig-src.json" }]
}
// @Filename: myproject/tsconfig-src.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target",
		"declarationMap": true,
	},
	"include": ["./src/\**/*"]
}
// @Filename: myproject/src/main.ts
import { foo } from './helpers/functions';
export { /*mainFoo*/foo };
// @Filename: myproject/src/helpers/functions.ts
export function foo() { return 1; }
// @Filename: myproject/indirect3/tsconfig.json
{ }
// @Filename: myproject/indirect3/main.ts
import { /*fooIndirect3Import*/foo } from '../target/src/main';
foo()
export function bar() {}
"#;
    let mut s = Session::new_for_test("findAllRefsSolutionWithDisableReferencedProjectLoadReferencingDefaultProjectDirectly", content);
    // TODO: // Ensure configured project is found for open file
    fourslash::go_to_marker(&mut s, "mainFoo");
    // TODO: // !!! TODO Verify errors
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: // Projects lifetime
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: // Find all refs in default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "mainFoo")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    // TODO: // Find all ref in non default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "fooIndirect3Import")
}

#[test]
fn find_all_refs_solution_referencing_default_project_indirectly_through_disable_referenced_project_load() {
    let content = r#"
// @stateBaseline: true 
// @tsc: --build /myproject/tsconfig.json
// @Filename: dummy/dummy.ts
/*dummy*/const x = 1;
// @Filename: dummy/tsconfig.json
{ }
// @Filename: myproject/tsconfig.json
{
	"files": [],
	"references":  [
		{ "path": "./tsconfig-indirect1.json" },
		{ "path": "./tsconfig-indirect2.json" },
	]
}
// @Filename: myproject/tsconfig-src.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target",
		"declarationMap": true,
	},
	"include": ["./src/\**/*"]
}
// @Filename: myproject/src/main.ts
import { foo } from './helpers/functions';
export { /*mainFoo*/foo };
// @Filename: myproject/src/helpers/functions.ts
export function foo() { return 1; }
// @Filename: myproject/indirect3/tsconfig.json
{ }
// @Filename: myproject/indirect3/main.ts
import { /*fooIndirect3Import*/foo } from '../target/src/main';
foo()
export function bar() {}
// @FileName: myproject/indirect1/main.ts
export const indirect = 1;
// @Filename: myproject/tsconfig-indirect1.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target/",
		"disableReferencedProjectLoad": true,
	},
	"files": [
		"./indirect1/main.ts"
	],
	"references": [
		{
			"path": "./tsconfig-src.json"
		}
	]
}
// @FileName: myproject/indirect2/main.ts
export const indirect = 1;
// @Filename: myproject/tsconfig-indirect2.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target/",
		"disableReferencedProjectLoad": true,
	},
	"files": [
		"./indirect2/main.ts"
	],
	"references": [
		{
			"path": "./tsconfig-src.json"
		}
	]
}
"#;
    let mut s = Session::new_for_test("findAllRefsSolutionReferencingDefaultProjectIndirectlyThroughDisableReferencedProjectLoad", content);
    // TODO: // Ensure configured project is found for open file
    fourslash::go_to_marker(&mut s, "mainFoo");
    // TODO: // !!! TODO Verify errors
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: // Projects lifetime
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: // Find all refs in default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "mainFoo")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    // TODO: // Find all ref in non default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "fooIndirect3Import")
}

#[test]
fn find_all_refs_solution_referencing_default_project_indirectly_through_disable_referenced_project_load_in_one_but_without_it_in_another() {
    let content = r#"
// @stateBaseline: true 
// @tsc: --build /myproject/tsconfig.json
// @Filename: dummy/dummy.ts
/*dummy*/const x = 1;
// @Filename: dummy/tsconfig.json
{ }
// @Filename: myproject/tsconfig.json
{
	"files": [],
	"references":  [
		{ "path": "./tsconfig-indirect1.json" },
		{ "path": "./tsconfig-indirect2.json" },
	]
}
// @Filename: myproject/tsconfig-src.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target",
		"declarationMap": true,
	},
	"include": ["./src/\**/*"]
}
// @Filename: myproject/src/main.ts
import { foo } from './helpers/functions';
export { /*mainFoo*/foo };
// @Filename: myproject/src/helpers/functions.ts
export function foo() { return 1; }
// @Filename: myproject/indirect3/tsconfig.json
{ }
// @Filename: myproject/indirect3/main.ts
import { /*fooIndirect3Import*/foo } from '../target/src/main';
foo()
export function bar() {}
// @FileName: myproject/indirect1/main.ts
export const indirect = 1;
// @Filename: myproject/tsconfig-indirect1.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target/",
		"disableReferencedProjectLoad": true,
	},
	"files": [
		"./indirect1/main.ts"
	],
	"references": [
		{
			"path": "./tsconfig-src.json"
		}
	]
}
// @FileName: myproject/indirect2/main.ts
export const indirect = 1;
// @Filename: myproject/tsconfig-indirect2.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target/",
	},
	"files": [
		"./indirect2/main.ts"
	],
	"references": [
		{
			"path": "./tsconfig-src.json"
		}
	]
}
"#;
    let mut s = Session::new_for_test("findAllRefsSolutionReferencingDefaultProjectIndirectlyThroughDisableReferencedProjectLoadInOneButWithoutItInAnother", content);
    // TODO: // Ensure configured project is found for open file
    fourslash::go_to_marker(&mut s, "mainFoo");
    // TODO: // !!! TODO Verify errors
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: // Projects lifetime
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: // Find all refs in default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "mainFoo")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    // TODO: // Find all ref in non default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "fooIndirect3Import")
}

#[test]
fn find_all_refs_project_with_own_files_referencing_file_from_referenced_project() {
    let content = r#"
// @stateBaseline: true 
// @tsc: --build /myproject/tsconfig.json
// @Filename: dummy/dummy.ts
/*dummy*/const x = 1;
// @Filename: dummy/tsconfig.json
{ }
// @Filename: myproject/tsconfig.json
{
	"files": ["./own/main.ts"],
	"references": [{ "path": "./tsconfig-src.json" }]
}
// @Filename: myproject/own/main.ts
import { foo } from '../target/src/main';
foo();
export function bar() {}
// @Filename: myproject/tsconfig-src.json
{
	"compilerOptions": {
		"composite": true,
		"outDir": "./target",
		"declarationMap": true,
	},
	"include": ["./src/\**/*"]
}
// @Filename: myproject/src/main.ts
import { foo } from './helpers/functions';
export { /*mainFoo*/foo };
// @Filename: myproject/src/helpers/functions.ts
export function foo() { return 1; }
// @Filename: myproject/indirect3/tsconfig.json
{ }
// @Filename: myproject/indirect3/main.ts
import { /*fooIndirect3Import*/foo } from '../target/src/main';
foo()
export function bar() {}
"#;
    let mut s = Session::new_for_test("findAllRefsProjectWithOwnFilesReferencingFileFromReferencedProject", content);
    // TODO: // Ensure configured project is found for open file
    fourslash::go_to_marker(&mut s, "mainFoo");
    // TODO: // !!! TODO Verify errors
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: // Projects lifetime
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    fourslash::go_to_marker(&mut s, "dummy");
    // TODO: f.CloseFileOfMarker(t, "dummy")
    // TODO: // Find all refs in default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "mainFoo")
    // TODO: f.CloseFileOfMarker(t, "mainFoo")
    // TODO: // Find all ref in non default project
    // TODO: f.VerifyBaselineFindAllReferences(t, "fooIndirect3Import")
}

#[test]
fn find_all_refs_overlapping_projects() {
    let content = r#"
// @stateBaseline: true 
// @Filename: solution/tsconfig.json
{
	"files": [],
	"include": [],
	"references": [
		{ "path": "./a" },
		{ "path": "./b" },
		{ "path": "./c" },
		{ "path": "./d" },
	],
}
// @Filename: solution/a/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
	"files": ["./index.ts"]
}
// @Filename: solution/a/index.ts
export interface I {
	M(): void;
}
// @Filename: solution/b/tsconfig.json
{
	"compilerOptions": {
		"composite": true
	},
	"files": ["./index.ts"],
	"references": [
		{ "path": "../a" },
	],
}
// @Filename: solution/b/index.ts
import { I } from "../a";
export class B implements /**/I {
	M() {}
}
// @Filename: solution/c/tsconfig.json
{
	"compilerOptions": {
		"composite": true
	},
	"files": ["./index.ts"],
	"references": [
		{ "path": "../b" },
	],
}
// @Filename: solution/c/index.ts
import { I } from "../a";
import { B } from "../b";
export const C: I = new B();
// @Filename: solution/d/tsconfig.json
{
	"compilerOptions": {
		"composite": true
	},
	"files": ["./index.ts"],
	"references": [
		{ "path": "../c" },
	],
}
// @Filename: solution/d/index.ts
import { I } from "../a";
import { C } from "../c";
export const D: I = C;
"#;
    let _s = Session::new_for_test("findAllRefsOverlappingProjects", content);
    // TODO: // The first search will trigger project loads
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
    // TODO: // The second search starts with the projects already loaded
    // TODO: // Formerly, this would search some projects multiple times
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}

#[test]
fn find_all_refs_two_projects_open_and_one_project_references() {
    let content = r#"
// @stateBaseline: true
// @Filename: /myproject/main/src/file1.ts
/*main*/export const mainConst = 10;
// @Filename: /myproject/main/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
	"references": [
		{ "path": "../core" },
		{ "path": "../indirect" },
		{ "path": "../noCoreRef1" },
		{ "path": "../indirectDisabledChildLoad1" },
		{ "path": "../indirectDisabledChildLoad2" },
		{ "path": "../refToCoreRef3" },
		{ "path": "../indirectNoCoreRef" }
	]
}
// @Filename: /myproject/core/src/file1.ts
export const /*find*/coreConst = 10;
// @Filename: /myproject/core/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
}
// @Filename: /myproject/noCoreRef1/src/file1.ts
export const noCoreRef1Const = 10;
// @Filename: /myproject/noCoreRef1/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
}
// @Filename: /myproject/indirect/src/file1.ts
export const indirectConst = 10;
// @Filename: /myproject/indirect/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
	"references": [
		{ "path": "../coreRef1" },
	]
}
// @Filename: /myproject/coreRef1/src/file1.ts
export const coreRef1Const = 10;
// @Filename: /myproject/coreRef1/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
	"references": [
		{ "path": "../core" },
	]
}
// @Filename: /myproject/indirectDisabledChildLoad1/src/file1.ts
export const indirectDisabledChildLoad1Const = 10;
// @Filename: /myproject/indirectDisabledChildLoad1/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
		"disableReferencedProjectLoad": true,
	},
	"references": [
		{ "path": "../coreRef2" },
	]
}
// @Filename: /myproject/coreRef2/src/file1.ts
export const coreRef2Const = 10;
// @Filename: /myproject/coreRef2/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
	"references": [
		{ "path": "../core" },
	]
}
// @Filename: /myproject/indirectDisabledChildLoad2/src/file1.ts
export const indirectDisabledChildLoad2Const = 10;
// @Filename: /myproject/indirectDisabledChildLoad2/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
		"disableReferencedProjectLoad": true,
	},
	"references": [
		{ "path": "../coreRef3" },
	]
}
// @Filename: /myproject/coreRef3/src/file1.ts
export const coreRef3Const = 10;
// @Filename: /myproject/coreRef3/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
	"references": [
		{ "path": "../core" },
	]
}
// @Filename: /myproject/refToCoreRef3/src/file1.ts
export const refToCoreRef3Const = 10;
// @Filename: /myproject/refToCoreRef3/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
	"references": [
		{ "path": "../coreRef3" },
	]
}
// @Filename: /myproject/indirectNoCoreRef/src/file1.ts
export const indirectNoCoreRefConst = 10;
// @Filename: /myproject/indirectNoCoreRef/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
	"references": [
		{ "path": "../noCoreRef2" },
	]
}
// @Filename: /myproject/noCoreRef2/src/file1.ts
export const noCoreRef2Const = 10;
// @Filename: /myproject/noCoreRef2/tsconfig.json
{
	"compilerOptions": {
		"composite": true,
	},
}"#;
    let mut s = Session::new_for_test("findAllRefsTwoProjectsOpenAndOneProjectReferences", content);
    fourslash::go_to_marker(&mut s, "main");
    // TODO: f.VerifyBaselineFindAllReferences(t, "find")
}

#[test]
fn find_all_refs_does_not_try_to_search_project_after_its_update_does_not_include_the_file() {
    let content = r#"
// @stateBaseline: true 
// @Filename: /packages/babel-loader/tsconfig.json
{
	"compilerOptions": {
		"target": "ES2018",
		"module": "commonjs",
		"strict": true,
		"esModuleInterop": true,
		"composite": true,
		"rootDir": "src",
		"outDir": "dist"
	},
	"include": ["src"],
	"references": [{"path": "../core"}]
}
// @Filename: /packages/babel-loader/src/index.ts
/*change*/import type { Foo } from "../../core/src/index.js";
// @Filename: /packages/core/tsconfig.json
{
	"compilerOptions": {
		"target": "ES2018",
		"module": "commonjs",
		"strict": true,
		"esModuleInterop": true,
		"composite": true,
		"rootDir": "./src",
		"outDir": "./dist",
	},
	"include": ["./src"]
}
// @Filename: /packages/core/src/index.ts
import { Bar } from "./loading-indicator.js";
export type Foo = {};
const bar: Bar = {
	/*prop*/prop: 0
}
// @Filename: /packages/core/src/loading-indicator.ts
export interface Bar {
	prop: number;
}
const bar: Bar = {
	prop: 1
}"#;
    let mut s = Session::new_for_test("findAllRefsDoesNotTryToSearchProjectAfterItsUpdateDoesNotIncludeTheFile", content);
    fourslash::go_to_marker(&mut s, "change");
    fourslash::go_to_marker(&mut s, "prop");
    // TODO: // Now change `babel-loader` project to no longer import `core` project
    fourslash::go_to_marker(&mut s, "change");
    fourslash::insert(&mut s, "// comment");
    // TODO: // At this point, we haven't updated `babel-loader` project yet,
    // TODO: // so `babel-loader` is still a containing project of `loading-indicator` file.
    // TODO: // When calling find all references,
    // TODO: // we shouldn't crash due to using outdated information on a file's containing projects.
    // TODO: f.VerifyBaselineFindAllReferences(t, "prop")
}

#[test]
fn find_all_refs_open_file_in_configured_project_that_will_be_removed() {
    let content = r#"
// @stateBaseline: true
// @Filename: /myproject/playground/tsconfig.json
{}
// @Filename: /myproject/playground/tests.ts
/*tests*/export function foo() {}
// @Filename: /myproject/playground/tsconfig-json/tsconfig.json
{
	"include": ["./src"]
}
// @Filename: /myproject/playground/tsconfig-json/src/src.ts
export function foobar() {}
// @Filename: /myproject/playground/tsconfig-json/tests/spec.ts
export function /*find*/bar() { }
"#;
    let mut s = Session::new_for_test("findAllRefsOpenFileInConfiguredProjectThatWillBeRemoved", content);
    fourslash::go_to_marker(&mut s, "tests");
    // TODO: f.CloseFileOfMarker(t, "tests")
    // TODO: f.VerifyBaselineFindAllReferences(t, "find")
}

#[test]
fn find_all_refs_re_export_in_multi_project_solution() {
    let content = r#"
// @stateBaseline: true
// @Filename: /tsconfig.base.json
{
	"compilerOptions": {
		"rootDir": ".",
		"outDir": "target",
		"module": "ESNext",
		"moduleResolution": "bundler",
		"composite": true,
		"declaration": true,
		"strict": true
	},
	"include": []
}
// @Filename: /tsconfig.json
{
	"extends": "./tsconfig.base.json",
	"references": [
		{ "path": "project-a" },
		{ "path": "project-b" },
		{ "path": "project-c" },
	]
}
// @Filename: /project-a/tsconfig.json
{
	"extends": "../tsconfig.base.json",
	"include": ["*"]
}
// @Filename: /project-a/private.ts
export const /*symbolA*/symbolA = 'some-symbol';
console.log(symbolA);
// @Filename: /project-a/public.ts
export { symbolA } from './private';
// @Filename: /project-b/tsconfig.json
{
	"extends": "../tsconfig.base.json",
	"include": ["*"]
}
// @Filename: /project-b/public.ts
export const /*symbolB*/symbolB = 'symbol-b';
// @Filename: /project-c/tsconfig.json
{
	"extends": "../tsconfig.base.json",
	"include": ["*"],
	"references": [
		{ "path": "../project-a" },
		{ "path": "../project-b" },
	]
}
// @Filename: /project-c/index.ts
import { symbolB } from '../project-b/public';
import { /*symbolAUsage*/symbolA } from '../project-a/public';
console.log(symbolB);
console.log(symbolA);
"#;
    let _s = Session::new_for_test("findAllRefsReExportInMultiProjectSolution", content);
    // TODO: // Find all refs for symbolA - should find definition in private.ts, re-export in public.ts, and usa
    // TODO: f.VerifyBaselineFindAllReferences(t, "symbolA")
    // TODO: // Find all refs for symbolB - should find definition and usage (no re-export involved)
    // TODO: f.VerifyBaselineFindAllReferences(t, "symbolB")
    // TODO: // Find all refs from the usage site - should also work
    // TODO: f.VerifyBaselineFindAllReferences(t, "symbolAUsage")
}

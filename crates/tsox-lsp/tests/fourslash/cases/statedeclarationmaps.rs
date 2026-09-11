use tsox_lsp::fourslash::{self, Session};


#[test]
fn declaration_maps_workspace_symbols() {
    let content = r#"// @stateBaseline: true
// @Filename: a/a.ts
export function fnA() {}
export interface IfaceA {}
export const instanceA: IfaceA = {};
// @Filename: a/tsconfig.json
{
	"compilerOptions": {
		"outDir": "bin",
		"declarationMap": true,
		"composite": true
	}
}
// @Filename: a/bin/a.d.ts.map
{
	"version": 3,
	"file": "a.d.ts",
	"sourceRoot": "",
	"sources": ["../a.ts"],
	"names": [],
	"mappings": "AAAA,wBAAgB,GAAG,SAAK;AACxB,MAAM,WAAW,MAAM;CAAG;AAC1B,eAAO,MAAM,SAAS,EAAE,MAAW,CAAC"
}
// @Filename: a/bin/a.d.ts
export declare function fnA(): void;
export interface IfaceA {
}
export declare const instanceA: IfaceA;
//# sourceMappingURL=a.d.ts.map
// @Filename: b/b.ts
export function fnB() {}
// @Filename: b/c.ts
export function fnC() {}
// @Filename: b/tsconfig.json
{
	"compilerOptions": {
		"outDir": "bin",
		"declarationMap": true,
		"composite": true
	}
}
// @Filename: b/bin/b.d.ts.map
{
	"version": 3,
	"file": "b.d.ts",
	"sourceRoot": "",
	"sources": ["../b.ts"],
	"names": [],
	"mappings": "AAAA,wBAAgB,GAAG,SAAK"
}
// @Filename: b/bin/b.d.ts
export declare function fnB(): void;
//# sourceMappingURL=b.d.ts.map
// @Filename: user/user.ts
/*user*/import * as a from "../a/a";
import * as b from "../b/b";
export function fnUser() {
	a.fnA();
	b.fnB();
	a.instanceA;
}
// @Filename: user/tsconfig.json
{
	"references": [
		{ "path": "../a" },
		{ "path": "../b" }
	]
}
// @Filename: dummy/dummy.ts
/*dummy*/export const a = 10;
// @Filename: dummy/tsconfig.json
{}"#;
    let mut s = Session::new_for_test("declarationMapsWorkspaceSymbols", content);
    fourslash::go_to_marker(&mut s, "user");
    // TODO: // Ref projects are loaded after as part of this command
    // TODO: f.VerifyBaselineWorkspaceSymbol(t, "fn")
    // TODO: // Open temp file and verify all projects alive
    // TODO: f.CloseFileOfMarker(t, "user")
    fourslash::go_to_marker(&mut s, "dummy");
}

#[test]
fn declaration_maps_find_all_refs_definition_in_mapped_file() {
    let content = r#"
// @stateBaseline: true 
//@Filename: a/a.ts
export function f() {}
// @Filename: a/tsconfig.json
{
	"compilerOptions": {
		"outDir": "../bin",
		"declarationMap": true,
		"composite": true
	}
}
//@Filename: b/b.ts
import { f } from "../bin/a";
/*1*/f();
// @Filename: b/tsconfig.json
{
	"references": [
		{ "path": "../a" }
	]
}
// @Filename: bin/a.d.ts
export declare function f(): void;
//# sourceMappingURL=a.d.ts.map
// @Filename: bin/a.d.ts.map
{
	"version":3,
	"file":"a.d.ts",
	"sourceRoot":"",
	"sources":["a.ts"],
	"names":[],
	"mappings":"AAAA,wBAAgB,CAAC,SAAK"
}"#;
    let mut s = Session::new_for_test("declarationMapsFindAllRefsDefinitionInMappedFile", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}

#[test]
fn declaration_maps_non_monotonic_mappings() {
    // TODO: // The source map creates a non-monotonic mapping:
    // TODO: // - .d.ts line 0 col 24 ('b' identifier) -> source line 1, col 16
    // TODO: // - .d.ts line 0 col 25 (right after 'b') -> source line 0, col 0 (EARLIER!)
    // TODO: //
    // TODO: // When looking up 'b' identifier [24, 25), start maps to ~byte 39,
    // TODO: // but end maps to byte 0, creating an inverted range.
    // TODO: // The fix in getMappedLocation clamps this to prevent negative ranges.
    let content = r#"
// @Filename: /src/index.ts
export function a() {}
export function b() {}
// @Filename: /src/indexdef.d.ts.map
{
	"version": 3,
	"file": "indexdef.d.ts",
	"sourceRoot": "",
	"sources": ["index.ts"],
	"names": [],
	"mappings": "AACA,wBAAgB,CADhB;AAAA,wBAAgB"
}
// @Filename: /src/indexdef.d.ts
export declare function b(): void;
export declare function a(): void;
//# sourceMappingURL=indexdef.d.ts.map
// @Filename: /src/user.ts
import { a, b } from "./indexdef";
/*1*/a();
/*2*/b();
// @Filename: /src/tsconfig.json
{}"#;
    let mut s = Session::new_for_test("declarationMapsNonMonotonicMappings", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1", "2")
}

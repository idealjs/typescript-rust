use tsox_lsp::fourslash::Session;


#[test]
fn go_to_source_node_modules_with_types() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/foo/package.json
{ "name": "foo", "version": "1.0.0", "main": "./lib/main.js", "types": "./types/main.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/foo/lib/main.js
export const /*end*/a = "a";
// @Filename: /home/src/workspaces/project/node_modules/foo/types/main.d.ts
export declare const a: string;
// @Filename: /home/src/workspaces/project/index.ts
import { a } from "foo";
[|a/*start*/|]"#;
    let _s = Session::new_for_test("goToSourceNodeModulesWithTypes", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "start")
}

#[test]
fn go_to_source_local_js_beside_dts() {
    let content = r#"// @Filename: /home/src/workspaces/project/a.js
export const /*end*/a = "a";
// @Filename: /home/src/workspaces/project/a.d.ts
export declare const a: string;
// @Filename: /home/src/workspaces/project/index.ts
import { a } from [|"./a"/*moduleSpecifier*/|];
[|a/*identifier*/|]"#;
    let _s = Session::new_for_test("goToSourceLocalJsBesideDts", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "identifier", "moduleSpecifier")
}

#[test]
fn go_to_source_non_declaration_file() {
    // TODO: // Declaration is in a .ts file (not .d.ts),
    // TODO: // so mapDeclarationToSourceDefinitions returns it as-is.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/utils.ts
export function /*target*/helper() { return 1; }
// @Filename: /home/src/workspaces/project/index.ts
import { helper } from "./utils";
helper/*usage*/();"#;
    let _s = Session::new_for_test("goToSourceNonDeclarationFile", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

#[test]
fn go_to_source_no_implementation_file() {
    // TODO: // No implementation file can be resolved (types-only package with no .js).
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function typesOnly(): void;
// @Filename: /home/src/workspaces/project/index.ts
import { /*importName*/typesOnly } from "pkg";
typesOnly/*callSite*/();"#;
    let _s = Session::new_for_test("goToSourceNoImplementationFile", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "importName", "callSite")
}

#[test]
fn go_to_source_declaration_map_source_map() {
    // TODO: // .d.ts has a sourcemap pointing back to the original .ts source.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./dist/index.js", "types": "./dist/index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/src/index.ts
export function /*target*/greet() { return "hi"; }
// @Filename: /home/src/workspaces/project/node_modules/pkg/dist/index.d.ts
export declare function greet(): string;
//# sourceMappingURL=index.d.ts.map
// @Filename: /home/src/workspaces/project/node_modules/pkg/dist/index.d.ts.map
{"version":3,"file":"index.d.ts","sourceRoot":"","sources":["../src/index.ts"],"names":[],"mappings":"AAAA,wBAAgB,KAAK,WAAY"}
// @Filename: /home/src/workspaces/project/node_modules/pkg/dist/index.js
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.greet = greet;
function greet() { return "hi"; }
// @Filename: /home/src/workspaces/project/index.ts
import { greet } from "pkg";
greet/*usage*/();"#;
    let _s = Session::new_for_test("goToSourceDeclarationMapSourceMap", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

#[test]
fn go_to_source_declaration_map_fallback() {
    // TODO: // findClosestDeclarationNode walks up parents and finds no declaration,
    // TODO: // returns entry node. This happens when source map points to a position
    // TODO: // that's not inside any declaration.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./dist/index.js", "types": "./dist/index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/src/index.ts
/*target*/console.log("side effect");
export function greet() { return "hi"; }
// @Filename: /home/src/workspaces/project/node_modules/pkg/dist/index.d.ts
export declare function greet(): string;
//# sourceMappingURL=index.d.ts.map
// @Filename: /home/src/workspaces/project/node_modules/pkg/dist/index.d.ts.map
{"version":3,"file":"index.d.ts","sourceRoot":"","sources":["../src/index.ts"],"names":[],"mappings":"AAC6B"}
// @Filename: /home/src/workspaces/project/node_modules/pkg/dist/index.js
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.greet = greet;
console.log("side effect");
function greet() { return "hi"; }
// @Filename: /home/src/workspaces/project/index.ts
import { greet } from "pkg";
greet/*usage*/();"#;
    let _s = Session::new_for_test("goToSourceDeclarationMapFallback", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

#[test]
fn go_to_source_named_exports_specifier() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function foo(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*target*/foo() { return "ok"; }
// @Filename: /home/src/workspaces/project/index.ts
import { foo } from "pkg";
const result = foo/*valueUsage*/();"#;
    let _s = Session::new_for_test("goToSourceNamedExportsSpecifier", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "valueUsage")
}

#[test]
fn go_to_source_triple_slash_reference() {
    // TODO: // Cursor on a /// <reference path="..."/> directive pointing to a .js file.
    let content = r#"// @allowJs: true
// @Filename: /home/src/workspaces/project/helper.js
/*target*/function helper() { return 1; }
// @Filename: /home/src/workspaces/project/index.ts
/// <reference path="./[|helper.js/*refPath*/|]" />
declare function helper(): number;
helper();"#;
    let _s = Session::new_for_test("goToSourceTripleSlashReference", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "refPath")
}

#[test]
fn go_to_source_fallback_to_module_specifier() {
    // TODO: // When the specific name can't be found in the .js implementation file
    // TODO: // (because the JS uses a different export pattern), the fallback returns
    // TODO: // the entry declaration of the .js file.
    let content = r#"// @moduleResolution: bundler
// @allowJs: true
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function internalHelper(): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
/*entryPoint*/Object.defineProperty(exports, "internalHelper", { value: function() {} });
// @Filename: /home/src/workspaces/project/index.ts
import { /*importName*/internalHelper } from "pkg";"#;
    let _s = Session::new_for_test("goToSourceFallbackToModuleSpecifier", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "importName")
}

#[test]
fn go_to_source_filter_preferred_fallback_all() {
    // TODO: // filterPreferredSourceDeclarations returns all declarations when none are
    // TODO: // property-like and none are concrete. This happens with re-export specifiers
    // TODO: // matching the name in the .js file.
    let content = r#"// @moduleResolution: bundler
// @allowJs: true
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./barrel.js", "types": "./barrel.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/barrel.d.ts
export { value } from "./impl";
// @Filename: /home/src/workspaces/project/node_modules/pkg/impl.d.ts
export declare const value: number;
// @Filename: /home/src/workspaces/project/node_modules/pkg/barrel.js
export { value } from "./impl.js";
// @Filename: /home/src/workspaces/project/node_modules/pkg/impl.js
export const /*target*/value = 42;
// @Filename: /home/src/workspaces/project/index.ts
import { /*importName*/value } from "pkg";
console.log(value);"#;
    let _s = Session::new_for_test("goToSourceFilterPreferredFallbackAll", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "importName")
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToSourceDefinition"]
#[test]
fn go_to_source_aliased_import_export() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare const foo: number;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
exports./*target*/foo = 1;
// @Filename: /home/src/workspaces/project/index.ts
import { foo as /*importAlias*/bar } from "pkg";
bar;
// @Filename: /home/src/workspaces/project/reexport.ts
export { foo as /*reExportAlias*/bar } from "pkg";"#;
    let mut s = Session::new_for_test("goToSourceAliasedImportExport", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "importAlias", "reExportAlias")
}

#[ignore = "generator: // import { original as alias } uses the propertyName branch"]
#[test]
fn go_to_source_aliased_import_specifier() {
    // TODO: // import { original as alias } uses the propertyName branch.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function original(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*target*/original() { return "ok"; }
// @Filename: /home/src/workspaces/project/index.ts
import { original as /*aliasedImport*/renamed } from "pkg";
renamed();"#;
    let mut s = Session::new_for_test("goToSourceAliasedImportSpecifier", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "aliasedImport")
}

#[ignore = "generator: // (in the current file) and the call signature target (from"]
#[test]
fn go_to_source_call_through_import() {
    // TODO: // When calling an imported function, the checker returns both the import specifier
    // TODO: // (in the current file) and the call signature target (from .d.ts → mapped to .js).
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare class Widget {
    constructor(name: string);
    render(): void;
}
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export class /*targetWidget*/Widget {
    constructor(name) { this.name = name; }
    /*targetRender*/render() {}
}
// @Filename: /home/src/workspaces/project/index.ts
import { Widget } from "pkg";
const w = new /*constructorCall*/Widget("test");
w./*methodCall*/render();"#;
    let mut s = Session::new_for_test("goToSourceCallThroughImport", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "constructorCall", "methodCall")
}

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToSourceDefinition"]
#[test]
fn go_to_source_callback_param() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/@types/yargs/package.json
{
    "name": "@types/yargs",
    "version": "1.0.0",
    "types": "./index.d.ts"
}
// @Filename: /home/src/workspaces/project/node_modules/@types/yargs/index.d.ts
export interface Yargs { positional(): Yargs; }
export declare function command(command: string, cb: (yargs: Yargs) => void): void;
// @Filename: /home/src/workspaces/project/node_modules/yargs/package.json
{
    "name": "yargs",
    "version": "1.0.0",
    "main": "index.js"
}
// @Filename: /home/src/workspaces/project/node_modules/yargs/index.js
export function command(cmd, cb) { cb({ /*end*/positional: "This is obviously not even close to realistic" }); }
// @Filename: /home/src/workspaces/project/index.ts
import { command } from "yargs";
command("foo", yargs => {
    yargs.[|/*start*/positional|]();
});"#;
    let mut s = Session::new_for_test("goToSourceCallbackParam", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "start")
}

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToSourceDefinition"]
#[test]
fn go_to_source_re_export_names() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function foo(): string;
export declare function bar(): number;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*targetFoo*/foo() { return "ok"; }
export function /*targetBar*/bar() { return 42; }
// @Filename: /home/src/workspaces/project/reexport.ts
export { /*reExportFoo*/foo, /*reExportBar*/bar } from "pkg";
// @Filename: /home/src/workspaces/project/index.ts
import { foo, bar } from [|"pkg"/*moduleSpecifier*/|];"#;
    let mut s = Session::new_for_test("goToSourceReExportNames", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "reExportFoo", "reExportBar", "moduleSpecifier")
}

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToSourceDefinition"]
#[test]
fn go_to_source_re_export_module_specifier() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function alpha(): string;
export declare function beta(): number;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*targetAlpha*/alpha() { return "a"; }
export function /*targetBeta*/beta() { return 2; }
// @Filename: /home/src/workspaces/project/reexport.ts
export { alpha, beta } from [|"pkg"/*reExportSpecifier*/|];"#;
    let mut s = Session::new_for_test("goToSourceReExportModuleSpecifier", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "reExportSpecifier")
}

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToSourceDefinition"]
#[test]
fn go_to_source_re_exported_implementation() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts", "type": "module" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export { foo } from "./foo";
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export { foo } from "./foo.js";
// @Filename: /home/src/workspaces/project/node_modules/pkg/foo.d.ts
export declare function foo(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/foo.js
export function /*target*/foo() { return "ok"; }
// @Filename: /home/src/workspaces/project/index.ts
import { /*importName*/foo } from "pkg";
foo/*start*/();"#;
    let mut s = Session::new_for_test("goToSourceReExportedImplementation", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "importName", "start")
}

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToSourceDefinition"]
#[test]
fn go_to_source_import_filtered_by_external_declaration() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function helper(): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*target*/helper() {}
// @Filename: /home/src/workspaces/project/index.ts
import { helper } from "pkg";
helper/*usage*/();
export { helper as /*reExport*/myHelper } from "pkg";"#;
    let mut s = Session::new_for_test("goToSourceImportFilteredByExternalDeclaration", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "usage", "reExport")
}

#[ignore = "generator: // The .d.ts declaration itself re-exports from another modu"]
#[test]
fn go_to_source_dts_re_export() {
    // TODO: // The .d.ts declaration itself re-exports from another module,
    // TODO: // so findContainingModuleSpecifier(declaration) finds that specifier.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/impl.d.ts
export declare function helper(): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/impl.js
export function /*target*/helper() {}
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export { helper } from "./impl";
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export { helper } from "./impl.js";
// @Filename: /home/src/workspaces/project/index.ts
import { helper } from "pkg";
helper/*usage*/();"#;
    let mut s = Session::new_for_test("goToSourceDtsReExport", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

#[ignore = "generator: // index.js re-exports from impl.js, causing getForwardedImp"]
#[test]
fn go_to_source_barrel_re_export_chain() {
    // TODO: // index.js re-exports from impl.js, causing getForwardedImplementationFiles
    // TODO: // to follow the chain.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/impl.js
export function /*target*/doWork() { return 42; }
// @Filename: /home/src/workspaces/project/node_modules/pkg/impl.d.ts
export declare function doWork(): number;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export { doWork } from "./impl";
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export { doWork } from "./impl.js";
// @Filename: /home/src/workspaces/project/index.ts
import { /*importName*/doWork } from "pkg";
doWork/*callSite*/();"#;
    let mut s = Session::new_for_test("goToSourceBarrelReExportChain", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "importName", "callSite")
}

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToSourceDefinition"]
#[test]
fn go_to_source_cjs_re_export_via_define_property() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function greet(name: string): string;
export declare enum TargetPopulation {
    Team = "team",
    Public = "public",
}
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TargetPopulation = exports.greet = void 0;
var impl_1 = require("./impl");
Object.defineProperty(exports, "greet", { enumerable: true, get: function () { return impl_1.greet; } });
var types_1 = require("./types");
Object.defineProperty(exports, "TargetPopulation", { enumerable: true, get: function () { return types_1.TargetPopulation; } });
// @Filename: /home/src/workspaces/project/node_modules/pkg/impl.js
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.greet = void 0;
function /*greetImpl*/greet(name) { return "Hello, " + name; }
exports.greet = greet;
// @Filename: /home/src/workspaces/project/node_modules/pkg/types.js
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TargetPopulation = void 0;
var /*targetPopulationImpl*/TargetPopulation;
(function (TargetPopulation) {
    TargetPopulation["Team"] = "team";
    TargetPopulation["Public"] = "public";
})(TargetPopulation || (exports.TargetPopulation = TargetPopulation = {}));
// @Filename: /home/src/workspaces/project/index.ts
import { /*namedImport*/greet, /*enumImport*/TargetPopulation } from "pkg";
greet/*call*/("world");
TargetPopulation/*enumAccess*/.Team;"#;
    let mut s = Session::new_for_test("goToSourceCJSReExportViaDefineProperty", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "namedImport", "enumImport", "call", "enumAccess")
}

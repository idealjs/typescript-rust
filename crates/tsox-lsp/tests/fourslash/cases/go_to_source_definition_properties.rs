use tsox_lsp::fourslash::Session;


#[test]
fn go_to_source_access_expression_property() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare const obj: { greet(name: string): string; count: number; };
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export const /*targetObj*/obj = { /*targetGreet*/greet(name) { return name; }, /*targetCount*/count: 42 };
// @Filename: /home/src/workspaces/project/index.ts
import { obj } from "pkg";
obj./*propAccess*/greet("world");
obj./*propAccess2*/count;"#;
    let _s = Session::new_for_test("goToSourceAccessExpressionProperty", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "propAccess", "propAccess2")
}

#[test]
fn go_to_source_property_of_alias() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/a.js
export const a = { /*end*/a: 'a' };
// @Filename: /home/src/workspaces/project/a.d.ts
export declare const a: { a: string };
// @Filename: /home/src/workspaces/project/b.ts
import { a } from './a';
a.[|a/*start*/|]"#;
    let _s = Session::new_for_test("goToSourcePropertyOfAlias", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "start")
}

#[test]
fn go_to_source_index_signature_property() {
    // TODO: // When accessing a property defined via index signature, getDeclarationsFromLocation
    // TODO: // returns empty, so the GetPropertyOfType fallback is used.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare const config: { readonly [key: string]: string; name: string };
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export const config = { /*targetName*/name: "test" };
// @Filename: /home/src/workspaces/project/index.ts
import { config } from "pkg";
config./*propAccess*/name;"#;
    let _s = Session::new_for_test("goToSourceIndexSignatureProperty", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "propAccess")
}

#[test]
fn go_to_source_mapped_type_property() {
    // TODO: // getDeclarationsFromLocation returns empty for a property that exists only
    // TODO: // via a mapped type (no explicit declaration), so GetPropertyOfType fallback is used.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
type Keys = "a" | "b";
export declare const obj: { [K in Keys]: number };
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export const obj = { a: 1, /*target*/b: 2 };
// @Filename: /home/src/workspaces/project/index.ts
import { obj } from "pkg";
obj./*propAccess*/b;"#;
    let _s = Session::new_for_test("goToSourceMappedTypeProperty", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "propAccess")
}

#[test]
fn go_to_source_common_js_alias_prefers_declaration() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare enum TargetPopulation {
    Team = "team",
    Internal = "internal",
    Insiders = "insider",
    Public = "public",
}
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TargetPopulation = void 0;
var TargetPopulation;
(function (TargetPopulation) {
    TargetPopulation["Team"] = "team";
    TargetPopulation["Internal"] = "internal";
    TargetPopulation["Insiders"] = "insider";
    TargetPopulation["Public"] = "public";
})(TargetPopulation || (exports.TargetPopulation = TargetPopulation = {}));
// @Filename: /home/src/workspaces/project/index.ts
import * as tas from "pkg";
tas./*start*/TargetPopulation.Public;"#;
    let _s = Session::new_for_test("goToSourceCommonJSAliasPrefersDeclaration", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "start")
}

use tsox_lsp::fourslash::Session;


#[test]
fn go_to_source_property_access_no_declaration() {
    // TODO: // When a property exists only via a mapped type in the .d.ts, the checker
    // TODO: // returns no declarations. The source definition resolver should still
    // TODO: // navigate to the property in the .js file by finding the module specifier
    // TODO: // from the parent expression's import declaration.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
type Keys = "alpha" | "beta";
export declare const config: { [K in Keys]: string };
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export const config = { /*targetAlpha*/alpha: "a", /*targetBeta*/beta: "b" };
// @Filename: /home/src/workspaces/project/index.ts
import { config } from "pkg";
config./*accessAlpha*/alpha;
config./*accessBeta*/beta;"#;
    let _s = Session::new_for_test("goToSourcePropertyAccessNoDeclaration", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "accessAlpha", "accessBeta")
}

#[test]
fn go_to_source_property_access_deep_chain() {
    // TODO: // Deep property access chain: import * as ns; ns.obj.prop
    // TODO: // where the intermediate object has no declaration but the root
    // TODO: // identifier can be traced back to its import.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare const nested: { inner: { value: number } };
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export const nested = { inner: { /*targetValue*/value: 42 } };
// @Filename: /home/src/workspaces/project/index.ts
import { nested } from "pkg";
nested.inner./*accessValue*/value;"#;
    let _s = Session::new_for_test("goToSourcePropertyAccessDeepChain", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "accessValue")
}

#[test]
fn go_to_source_property_access_namespace_import() {
    // TODO: // import * as ns from "pkg"; ns.thing — where "thing" has no declarations
    // TODO: // from the checker (e.g. module augmentation or dynamic).
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
type Keys = "x" | "y";
export declare const coords: { [K in Keys]: number };
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export const coords = { /*targetX*/x: 10, /*targetY*/y: 20 };
// @Filename: /home/src/workspaces/project/index.ts
import { coords } from "pkg";
coords./*accessX*/x;
coords./*accessY*/y;"#;
    let _s = Session::new_for_test("goToSourcePropertyAccessNamespaceImport", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "accessX", "accessY")
}

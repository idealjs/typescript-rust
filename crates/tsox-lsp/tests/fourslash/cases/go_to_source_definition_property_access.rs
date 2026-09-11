use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_source_mapped_type_property_with_match() {
    // TODO: // When accessing a property that only exists via a mapped type, the checker
    // TODO: // returns no declarations. The property access fallback (GetPropertyOfType)
    // TODO: // should find the property if it's in the .js implementation file.
    // TODO: // This test differs from the existing goToSourceMappedTypeProperty by having
    // TODO: // a named explicit property in the .d.ts alongside the mapped type.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare const obj: { a: number; b: number };
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export const obj = { /*targetA*/a: 1, /*targetB*/b: 2 };
// @Filename: /home/src/workspaces/project/index.ts
import { obj } from "pkg";
obj./*propA*/a;
obj./*propB*/b;"#;
    let mut s = Session::new_for_test("goToSourceMappedTypePropertyWithMatch", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "propA", "propB")
}

#[test]
fn go_to_source_namespace_import_property() {
    // TODO: // import * as ns from "pkg"; ns.prop — should navigate to the property
    // TODO: // in the .js file.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function helper(): void;
export declare const value: number;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*targetHelper*/helper() {}
export const /*targetValue*/value = 42;
// @Filename: /home/src/workspaces/project/index.ts
import * as pkg from "pkg";
pkg./*helperAccess*/helper();
pkg./*valueAccess*/value;"#;
    let mut s = Session::new_for_test("goToSourceNamespaceImportProperty", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "helperAccess", "valueAccess")
}

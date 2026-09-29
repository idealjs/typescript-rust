use tsox_lsp::fourslash::Session;


#[test]
fn go_to_source_aliased_import_at_usage_site() {
    // TODO: // When the cursor is on a usage of an aliased import (not on the import
    // TODO: // findImportForName, and the original export name is passed as
    // TODO: // additionalNames so that the .js file is searched for the correct
    // TODO: // declaration. Without additionalNames, only the alias name would be
    // TODO: // searched, which does not exist in the .js file.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function unrelated(): void;
export declare function original(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function unrelated() {}
export function /*target*/original() { return "ok"; }
// @Filename: /home/src/workspaces/project/index.ts
import { original as renamed } from "pkg";
renamed/*usage*/();"#;
    let _s = Session::new_for_test("goToSourceAliasedImportAtUsageSite", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

#[test]
fn go_to_source_aliased_import_at_usage_site_namespace_import() {
    // TODO: // When an aliased namespace import (import * as ns) is used at a property
    // TODO: // access site (ns.foo), the root identifier's import is discovered and the
    // TODO: // module specifier is used to resolve the property in the .js file.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function helper(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*target*/helper() { return "ok"; }
// @Filename: /home/src/workspaces/project/index.ts
import * as ns from "pkg";
ns./*usage*/helper();"#;
    let _s = Session::new_for_test("goToSourceAliasedImportAtUsageSiteNamespaceImport", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // When importing { original as alias }, the module specifie"]
#[test]
fn go_to_source_aliased_import_with_preceding_exports() {
    // TODO: // When importing { original as alias }, the module specifier path resolves
    // TODO: // the alias text (not the original name). If the target function is NOT the
    // TODO: // first export in the .js file, the entry-declaration fallback will point to
    // TODO: // the wrong declaration. The fix should resolve to the original export name.
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
import { original as /*aliasedImport*/renamed } from "pkg";
renamed();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "aliasedImport")
}

#[ignore = "generator: // Re-export with alias: export { original as alias } from '"]
#[test]
fn go_to_source_re_export_alias_with_preceding_exports() {
    // TODO: // Re-export with alias: export { original as alias } from "pkg"
    // TODO: // should navigate to the original export, not the first statement.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function unrelated(): void;
export declare function original(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function unrelated() {}
export function /*target*/original() { return "ok"; }
// @Filename: /home/src/workspaces/project/reexport.ts
export { original as /*reExportAlias*/renamed } from "pkg";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "reExportAlias")
}

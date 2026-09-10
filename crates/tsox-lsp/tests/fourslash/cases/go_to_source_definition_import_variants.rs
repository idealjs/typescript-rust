use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // findContainingModuleSpecifier handles require() calls."]
#[test]
fn go_to_source_require_call() {
    // TODO: // findContainingModuleSpecifier handles require() calls.
    let content = r#"// @moduleResolution: bundler
// @allowJs: true
// @checkJs: true
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function helper(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
exports./*target*/helper = function() { return "ok"; };
// @Filename: /home/src/workspaces/project/index.js
const { /*importName*/helper } = require("pkg");
helper/*usage*/();"#;
    let mut s = Session::new_for_test("goToSourceRequireCall", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "importName", "usage")
}

#[ignore = "generator: // findContainingModuleSpecifier handles dynamic import() ca"]
#[test]
fn go_to_source_dynamic_import() {
    // TODO: // findContainingModuleSpecifier handles dynamic import() calls.
    let content = r#"// @moduleResolution: bundler
// @target: esnext
// @module: esnext
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function dynHelper(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*target*/dynHelper() { return "dynamic"; }
// @Filename: /home/src/workspaces/project/index.ts
async function main() {
    const mod = await import("pkg");
    mod./*usage*/dynHelper();
}"#;
    let mut s = Session::new_for_test("goToSourceDynamicImport", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

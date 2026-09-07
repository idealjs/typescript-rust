use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToSourceDefinition"]
#[test]
fn go_to_source_fallbacks_to_definition_for_interface() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export interface /*target*/Config {
    enabled: boolean;
}
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
exports.makeConfig = () => ({ enabled: true });
// @Filename: /home/src/workspaces/project/index.ts
import type { /*importName*/Config } from "pkg";
let value: /*typeRef*/Config;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "importName", "typeRef")
}

#[ignore = "generator: // (the .d.ts declaration) since there's no concrete JS impl"]
#[test]
fn go_to_source_type_only_symbol_fallback() {
    // TODO: // When a type-only symbol (type alias) is imported with a regular import and used
    // TODO: // in a value position, source definition should fall back to regular definition
    // TODO: // (the .d.ts declaration) since there's no concrete JS implementation.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/types.d.ts
export interface Config { enabled: boolean; }
// @Filename: /home/src/workspaces/project/node_modules/pkg/types.js
// no runtime content for Config interface
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export { Config } from "./types";
export declare function makeConfig(): Config;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export { Config } from "./types.js";
export function makeConfig() { return { enabled: true }; }
// @Filename: /home/src/workspaces/project/index.ts
import { Config, makeConfig } from "pkg";
let c: /*typeRef*/Config;
makeConfig/*callRef*/();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "typeRef", "callRef")
}

#[ignore = "generator: // Forwarded declarations are non-concrete, so they merge wi"]
#[test]
fn go_to_source_forwarded_non_concrete_merge() {
    // TODO: // Forwarded declarations are non-concrete, so they merge with the initial
    // TODO: // non-concrete declarations. The barrel index.js re-exports from types.js
    // TODO: // which only has type re-exports.
    let content = r#"// @moduleResolution: bundler
// @allowJs: true
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export { Config } from "./types";
// @Filename: /home/src/workspaces/project/node_modules/pkg/types.d.ts
export interface Config { enabled: boolean; }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export { Config } from "./types.js";
// @Filename: /home/src/workspaces/project/node_modules/pkg/types.js
// Config is a type, no runtime value
// @Filename: /home/src/workspaces/project/index.ts
import { /*importName*/Config } from "pkg";
let c: Config;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "importName")
}

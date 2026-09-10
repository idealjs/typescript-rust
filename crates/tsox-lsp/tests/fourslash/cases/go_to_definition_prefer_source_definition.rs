use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // 1. Regular go-to-definition: goes to the .d.ts file"]
#[test]
fn go_to_definition_prefer_source_definition() {
    let content = r#"// @Filename: /home/src/workspaces/project/a.js
export const /*sourceTarget*/a = "a";
// @Filename: /home/src/workspaces/project/a.d.ts
export declare const /*dtsTarget*/a: string;
// @Filename: /home/src/workspaces/project/index.ts
import { a } from "./a";
a/*start*/"#;
    let mut s = Session::new_for_test("goToDefinitionPreferSourceDefinition", content);
    // TODO: // 1. Regular go-to-definition: goes to the .d.ts file
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false /*includeOriginalSelectionRange*/, "start")
    // TODO: // 2. Go-to-source-definition: goes to the .js file
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "start")
    // TODO: // 3. Go-to-definition with preferGoToSourceDefinition: goes to the .js file, same as source definit
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{PreferGoToSourceDefinition: true})
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false /*includeOriginalSelectionRange*/, "start")
}

#[ignore = "generator: // With preferGoToSourceDefinition, when no source .js defin"]
#[test]
fn go_to_definition_prefer_source_definition_fallback() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export interface Config {
    enabled: boolean;
}
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
exports.makeConfig = () => ({ enabled: true });
// @Filename: /home/src/workspaces/project/index.ts
import type { Config } from "pkg";
let value: /*start*/Config;"#;
    let mut s = Session::new_for_test("goToDefinitionPreferSourceDefinitionFallback", content);
    // TODO: // With preferGoToSourceDefinition, when no source .js definition exists for a type-only symbol,
    // TODO: // go-to-definition should fall back to the .d.ts definition.
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{PreferGoToSourceDefinition: true})
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false /*includeOriginalSelectionRange*/, "start")
}

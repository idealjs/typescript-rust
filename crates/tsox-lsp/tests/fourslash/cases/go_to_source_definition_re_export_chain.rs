use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // When an implementation file re-exports from another file,"]
#[test]
fn go_to_source_forwarded_re_export_chain() {
    // TODO: // When an implementation file re-exports from another file, source
    // TODO: // definition follows the re-export chain to the actual implementation.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function helper(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export { helper } from './impl.js';
// @Filename: /home/src/workspaces/project/node_modules/pkg/impl.js
export function /*targetHelper*/helper() { return "ok"; }
// @Filename: /home/src/workspaces/project/index.ts
import { /*importHelper*/helper } from "pkg";
helper();"#;
    let mut s = Session::new_for_test("goToSourceForwardedReExportChain", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "importHelper")
}

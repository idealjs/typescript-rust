use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // When a symbol has merged declarations (class + namespace)"]
#[test]
fn go_to_source_merged_declaration_dedup() {
    // TODO: // When a symbol has merged declarations (class + namespace), source
    // TODO: // definition deduplicates them and navigates to the single source class.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare class /*dtsClass*/Util {
    run(): void;
}
export declare namespace Util {
    export const version: string;
}
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export class /*targetUtil*/Util {
    run() {}
}
Util.version = "1.0";
// @Filename: /home/src/workspaces/project/index.ts
import { /*importUtil*/Util } from "pkg";
const u: /*typeRef*/Util = new Util();"#;
    let mut s = Session::new_for_test("goToSourceMergedDeclarationDedup", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "importUtil", "typeRef")
}

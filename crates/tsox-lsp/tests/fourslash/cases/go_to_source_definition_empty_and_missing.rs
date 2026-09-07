use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // When the resolved .js file is empty (0 statements), sourc"]
#[test]
fn go_to_source_definition_empty_js_file() {
    // TODO: // When the resolved .js file is empty (0 statements), source definition
    // TODO: // navigates to the SourceFile node itself.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function foo(): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
// @Filename: /home/src/workspaces/project/index.ts
import { foo } from /*specifier*/"pkg";
foo();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "specifier")
}

#[ignore = "generator: // When a default import resolves to a .js file that has no "]
#[test]
fn go_to_source_default_import_no_default_in_js() {
    // TODO: // When a default import resolves to a .js file that has no default export,
    // TODO: // source definition falls back to the first statement of the file.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export default function create(): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
/*targetEntry*/function internalCreate() { return {}; }
module.exports = { create: internalCreate };
// @Filename: /home/src/workspaces/project/index.ts
import /*importDefault*/create from "pkg";
create();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "importDefault")
}

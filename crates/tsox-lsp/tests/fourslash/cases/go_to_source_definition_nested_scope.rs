use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // a function body. We should navigate to the exported funct"]
#[test]
fn go_to_source_nested_scope_shadowing() {
    // TODO: // findDeclarationNodesByName should only match top-level/exported declarations,
    // TODO: // not nested locals that happen to share the same name. Here "helper" is
    // TODO: // exported at the top level, but there's also a local "helper" variable inside
    // TODO: // a function body. We should navigate to the exported function, not the local.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function helper(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*targetHelper*/helper() { return "ok"; }
function unrelated() {
    const helper = "shadow";
    return helper;
}
// @Filename: /home/src/workspaces/project/index.ts
import { /*importHelper*/helper } from "pkg";
helper/*usage*/();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "importHelper", "usage")
}

#[ignore = "generator: // A class 'Widget' is exported at the top level, and there'"]
#[test]
fn go_to_source_nested_class_shadowing() {
    // TODO: // A class "Widget" is exported at the top level, and there's also a local
    // TODO: // class "Widget" inside a function. We should only navigate to the exported one.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare class Widget {}
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export class /*targetWidget*/Widget {}
function factory() {
    class Widget { constructor() { this.local = true; } }
    return new Widget();
}
// @Filename: /home/src/workspaces/project/index.ts
import { /*importWidget*/Widget } from "pkg";
new Widget();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "importWidget")
}

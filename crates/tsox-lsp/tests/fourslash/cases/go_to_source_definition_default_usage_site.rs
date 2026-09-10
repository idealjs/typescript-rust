use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // When the cursor is on a usage of a default import (not on"]
#[test]
fn go_to_source_default_import_usage_site_checker() {
    // TODO: // When the cursor is on a usage of a default import (not on the import
    // TODO: // must include "default" from the resolved declaration's export-default
    // TODO: // modifier, since isDefaultImportName returns false at the usage site.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export default class Widget {
    render(): void;
}
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export default class /*targetWidget*/Widget {
    /*targetRender*/render() {}
}
// @Filename: /home/src/workspaces/project/index.ts
import Widget from "pkg";
const w = new Widget/*constructUsage*/("test");
w./*methodUsage*/render();"#;
    let mut s = Session::new_for_test("goToSourceDefaultImportUsageSiteChecker", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "constructUsage", "methodUsage")
}

#[ignore = "generator: // Default import re-exported and then used at a call site. "]
#[test]
fn go_to_source_default_import_re_export_usage() {
    // TODO: // Default import re-exported and then used at a call site. The checker
    // TODO: // must resolve the alias chain, and the source definition should reach
    // TODO: // the original implementation file.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export default function greet(name: string): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export default function /*targetGreet*/greet(name) { return "Hello, " + name; }
// @Filename: /home/src/workspaces/project/index.ts
import greet from "pkg";
greet/*callUsage*/("world");"#;
    let mut s = Session::new_for_test("goToSourceDefaultImportReExportUsage", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "callUsage")
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_source_scoped_package() {
    // TODO: // Scoped packages (@scope/pkg) exercise UnmangleScopedPackageName
    // TODO: // in findImplementationFileFromDtsFileName.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/@myscope/mylib/package.json
{ "name": "@myscope/mylib", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/@myscope/mylib/index.d.ts
export declare function scopedHelper(): string;
// @Filename: /home/src/workspaces/project/node_modules/@myscope/mylib/index.js
export function /*target*/scopedHelper() { return "scoped"; }
// @Filename: /home/src/workspaces/project/index.ts
import { /*importName*/scopedHelper } from "@myscope/mylib";
scopedHelper/*usage*/();"#;
    let mut s = Session::new_for_test("goToSourceScopedPackage", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "importName", "usage")
}

#[test]
fn go_to_source_scoped_at_types_package() {
    // TODO: // @types/@scope/pkg should map to @scope/pkg implementation.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/@types/myns__mylib/package.json
{ "name": "@types/myns__mylib", "version": "1.0.0" }
// @Filename: /home/src/workspaces/project/node_modules/@types/myns__mylib/index.d.ts
export declare function nsHelper(): number;
// @Filename: /home/src/workspaces/project/node_modules/@myns/mylib/package.json
{ "name": "@myns/mylib", "version": "1.0.0", "main": "./index.js" }
// @Filename: /home/src/workspaces/project/node_modules/@myns/mylib/index.js
export function /*target*/nsHelper() { return 42; }
// @Filename: /home/src/workspaces/project/index.ts
import { nsHelper } from "@myns/mylib";
nsHelper/*usage*/();"#;
    let mut s = Session::new_for_test("goToSourceScopedAtTypesPackage", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // /// <reference types='foo'/> resolves to @types/foo/index"]
#[test]
fn go_to_source_reference_types_to_js() {
    // TODO: // /// <reference types="foo"/> resolves to @types/foo/index.d.ts.
    // TODO: // Source definition should find the corresponding foo/index.js.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/@types/foo/package.json
{ "name": "@types/foo", "version": "1.0.0" }
// @Filename: /home/src/workspaces/project/node_modules/@types/foo/index.d.ts
export declare function bar(): string;
// @Filename: /home/src/workspaces/project/node_modules/foo/package.json
{ "name": "foo", "version": "1.0.0", "main": "./index.js" }
// @Filename: /home/src/workspaces/project/node_modules/foo/index.js
export function /*target*/bar() { return "hello"; }
// @Filename: /home/src/workspaces/project/index.ts
/// <reference types="[|foo/*refTypes*/|]" />
import { bar } from "foo";
bar();"#;
    let mut s = Session::new_for_test("goToSourceReferenceTypesToJS", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "refTypes")
}

#[ignore = "generator: // /// <reference path='./lib.d.ts'/> where a sibling .js fi"]
#[test]
fn go_to_source_reference_path_to_dts() {
    // TODO: // /// <reference path="./lib.d.ts"/> where a sibling .js file exists.
    // TODO: // Source definition should navigate to the .js implementation.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
/// <reference path="./lib.d.ts" />
export declare function main(): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/lib.d.ts
export declare function helper(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/lib.js
export function /*target*/helper() { return "ok"; }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function main() {}
// @Filename: /home/src/workspaces/project/index.ts
/// <reference path="./node_modules/pkg/[|lib.d.ts/*refPath*/|]" />
declare function helper(): string;"#;
    let mut s = Session::new_for_test("goToSourceReferencePathToDts", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "refPath")
}

use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // NoDts resolver can't resolve 'foo' to any .js (only @type"]
#[test]
fn go_to_source_at_types_package() {
    // TODO: // NoDts resolver can't resolve "foo" to any .js (only @types/foo has .d.ts),
    // TODO: // so findImplementationFileFromDtsFileName maps @types/foo → foo and finds the .js.
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
import { bar } from "foo";
bar/*usage*/();"#;
    let mut s = Session::new_for_test("goToSourceAtTypesPackage", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

#[ignore = "generator: // When the .d.ts is index.d.ts, tryPackageRootFirst is true"]
#[test]
fn go_to_source_package_index_dts() {
    // TODO: // When the .d.ts is index.d.ts, tryPackageRootFirst is true,
    // TODO: // so package root resolution is tried before subpath.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./lib/index.js", "types": "./lib/index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/lib/index.d.ts
export declare function greet(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/lib/index.js
export function /*target*/greet() { return "hi"; }
// @Filename: /home/src/workspaces/project/index.ts
import { greet } from "pkg";
greet/*usage*/();"#;
    let mut s = Session::new_for_test("goToSourcePackageIndexDts", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

#[ignore = "generator: // there's no main entry, but subpath resolution ('pkg/index"]
#[test]
fn go_to_source_package_root_then_subpath() {
    // TODO: // tryPackageRootFirst is true (index.d.ts), root resolution fails because
    // TODO: // there's no main entry, but subpath resolution ("pkg/index") succeeds.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function work(): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*target*/work() {}
// @Filename: /home/src/workspaces/project/index.ts
import { work } from "pkg";
work/*usage*/();"#;
    let mut s = Session::new_for_test("goToSourcePackageRootThenSubpath", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

#[ignore = "generator: // falls back to subpath."]
#[test]
fn go_to_source_package_root_falls_back_to_subpath() {
    // TODO: // tryPackageRootFirst is true (index.d.ts), root resolution fails,
    // TODO: // falls back to subpath.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function work(): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*target*/work() {}
// @Filename: /home/src/workspaces/project/index.ts
import { work } from "pkg";
work/*usage*/();"#;
    let mut s = Session::new_for_test("goToSourcePackageRootFallsBackToSubpath", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

#[ignore = "generator: // Subpath resolution succeeds when the d.ts is NOT index.d."]
#[test]
fn go_to_source_subpath_not_index() {
    // TODO: // Subpath resolution succeeds when the d.ts is NOT index.d.ts.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "types": "./lib/utils.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/lib/utils.d.ts
export declare function util(): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/lib/utils.js
export function /*target*/util() {}
// @Filename: /home/src/workspaces/project/index.ts
import { util } from "pkg";
util/*usage*/();"#;
    let mut s = Session::new_for_test("goToSourceSubpathNotIndex", content);
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

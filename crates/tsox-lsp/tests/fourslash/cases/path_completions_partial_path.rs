use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_partial_path_relative_import() {
    let content = r#"// @Filename: /src/main.ts
import { } from "./foo//*$*/";
// @Filename: /src/foo/async.ts
export const asyncApi = "async";
// @Filename: /src/foo/fs.ts
export const fsApi = "fs";
// @Filename: /src/foo/sync.ts
export const syncApi = "sync";"#;
    let mut s = Session::new_for_test("pathCompletionsPartialPathRelativeImport", content);
    fourslash::verify_completions_exact_at(&mut s, Some("$"), &["async", "fs", "sync"]);
}

#[test]
fn path_completions_partial_path_package_no_exports() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /node_modules/@typescript/typescript/package.json
{ "name": "@typescript/typescript", "version": "0.0.0" }
// @Filename: /node_modules/@typescript/typescript/unstable/async.ts
export const asyncApi = "async";
// @Filename: /node_modules/@typescript/typescript/unstable/fs.ts
export const fsApi = "fs";
// @Filename: /node_modules/@typescript/typescript/unstable/sync.ts
export const syncApi = "sync";
// @Filename: /package.json
{ "dependencies": { "@typescript/typescript": "0.0.0" } }
// @Filename: /src/main.ts
import { } from "@typescript/typescript/unstable//*$*/";"#;
    let mut s = Session::new_for_test("pathCompletionsPartialPathPackageNoExports", content);
    fourslash::verify_completions_exact_at(&mut s, Some("$"), &["async", "fs", "sync"]);
}

#[test]
fn path_completions_partial_path_package_exports() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /node_modules/@typescript/typescript/package.json
{
	"name": "@typescript/typescript",
	"version": "0.0.0",
	"exports": {
		"./unstable/sync": "./dist/api/sync/api.js",
		"./unstable/async": "./dist/api/async/api.js",
		"./unstable/fs": "./dist/api/fs.js"
	}
}
// @Filename: /node_modules/@typescript/typescript/index.d.ts
export {};
// @Filename: /node_modules/@typescript/typescript/dist/api/async/api.js
export const asyncApi = "async";
// @Filename: /node_modules/@typescript/typescript/dist/api/fs.js
export const fsApi = "fs";
// @Filename: /node_modules/@typescript/typescript/dist/api/sync/api.js
export const syncApi = "sync";
// @Filename: /package.json
{ "dependencies": { "@typescript/typescript": "0.0.0" } }
// @Filename: /src/main.ts
import { } from "@typescript/typescript/unstable//*$*/";"#;
    let mut s = Session::new_for_test("pathCompletionsPartialPathPackageExports", content);
    fourslash::verify_completions_exact_at(&mut s, Some("$"), &["async", "fs", "sync"]);
}

#[test]
fn path_completions_partial_path_package_exports_ending_star() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /node_modules/@typescript/typescript/package.json
{
	"name": "@typescript/typescript",
	"version": "0.0.0",
	"exports": {
		"./unstable/*": "./dist/unstable/*.d.ts"
	}
}
// @Filename: /node_modules/@typescript/typescript/dist/unstable/async.d.ts
export declare const asyncApi: string;
// @Filename: /node_modules/@typescript/typescript/dist/unstable/fs.d.ts
export declare const fsApi: string;
// @Filename: /node_modules/@typescript/typescript/dist/unstable/sync.d.ts
export declare const syncApi: string;
// @Filename: /package.json
{ "dependencies": { "@typescript/typescript": "0.0.0" } }
// @Filename: /src/main.ts
import { } from "@typescript/typescript/unstable//*$*/";"#;
    let mut s = Session::new_for_test("pathCompletionsPartialPathPackageExportsEndingStar", content);
    fourslash::verify_completions_exact_at(&mut s, Some("$"), &["async", "fs", "sync"]);
}

#[test]
fn path_completions_partial_path_package_exports_middle_star() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /node_modules/@typescript/typescript/package.json
{
	"name": "@typescript/typescript",
	"version": "0.0.0",
	"exports": {
		"./unstable/_*/api": "./dist/api/*.d.ts"
	}
}
// @Filename: /node_modules/@typescript/typescript/dist/api/async.d.ts
export declare const asyncApi: string;
// @Filename: /node_modules/@typescript/typescript/dist/api/fs.d.ts
export declare const fsApi: string;
// @Filename: /node_modules/@typescript/typescript/dist/api/sync.d.ts
export declare const syncApi: string;
// @Filename: /package.json
{ "dependencies": { "@typescript/typescript": "0.0.0" } }
// @Filename: /src/main.ts
import { } from "@typescript/typescript/unstable//*$*/";"#;
    let mut s = Session::new_for_test("pathCompletionsPartialPathPackageExportsMiddleStar", content);
    fourslash::verify_completions_exact_at(&mut s, Some("$"), &["_async/api", "_fs/api", "_sync/api"]);
}

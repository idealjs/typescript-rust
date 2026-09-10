use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_import_duplicate_packages_scoped() {
    let content = r#"// @lib: es5
// @module: commonjs
// @esModuleInterop: true
// @Filename: /node_modules/@scope/react-dom/package.json
{ "name": "react-dom", "version": "1.0.0", "types": "./index.d.ts" }
// @Filename: /node_modules/@scope/react-dom/index.d.ts
import * as React from "react";
export function render(): void;
// @Filename: /node_modules/@scope/react/package.json
{ "name": "react", "version": "1.0.0", "types": "./index.d.ts" }
// @Filename: /node_modules/@scope/react/index.d.ts
import "./other";
export declare function useState(): void;
// @Filename: /node_modules/@scope/react/other.d.ts
export declare function useRef(): void;
// @Filename: /packages/a/node_modules/@scope/react/package.json
{ "name": "react", "version": "1.0.1", "types": "./index.d.ts" }
// @Filename: /packages/a/node_modules/@scope/react/index.d.ts
export declare function useState(): void;
// @Filename: /packages/a/index.ts
import "@scope/react-dom";
import "@scope/react";
// @Filename: /packages/a/foo.ts
/**/"#;
    let mut s = Session::new_for_test("completionsImport_duplicatePackages_scoped", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

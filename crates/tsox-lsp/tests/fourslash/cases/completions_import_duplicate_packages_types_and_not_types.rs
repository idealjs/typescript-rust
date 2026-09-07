use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_import_duplicate_packages_types_and_not_types() {
    let content = r#"// @lib: es5
// @module: commonjs
// @esModuleInterop: true
// @Filename: /node_modules/@types/react-dom/package.json
{ "name": "react-dom", "version": "1.0.0", "types": "./index.d.ts" }
// @Filename: /node_modules/@types/react-dom/index.d.ts
import * as React from "react";
export function render(): void;
// @Filename: /node_modules/@types/react/package.json
{ "name": "react", "version": "1.0.0", "types": "./index.d.ts" }
// @Filename: /node_modules/@types/react/index.d.ts
import "./other";
export declare function useState(): void;
// @Filename: /node_modules/@types/react/other.d.ts
export declare function useRef(): void;
// @Filename: /packages/a/node_modules/react/package.json
{ "name": "react", "version": "1.0.1", "types": "./index.d.ts" }
// @Filename: /packages/a/node_modules/react/index.d.ts
export declare function useState(): void;
// @Filename: /packages/a/index.ts
import "react-dom";
import "react";
// @Filename: /packages/a/foo.ts
useState/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

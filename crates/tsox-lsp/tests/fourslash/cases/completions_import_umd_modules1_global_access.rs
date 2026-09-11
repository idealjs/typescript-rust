use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_umd_modules1_global_access() {
    let content = r#"// @filename: /package.json
{ "dependencies": { "@types/classnames": "*" } }
// @filename: /tsconfig.json
{ "compilerOptions": { "allowUmdGlobalAccess": true, "types": ["*"] } }
// @filename: /node_modules/@types/classnames/package.json
{ "name": "@types/classnames", "types": "index.d.ts" }
// @filename: /node_modules/@types/classnames/index.d.ts
declare const classNames: () => string;
export = classNames;
export as namespace classNames;
// @filename: /SomeReactComponent.tsx
import * as React from 'react';

const el1 = <div className={class/*1*/}>foo</div>;"#;
    let mut s = Session::new_for_test("completionsImport_umdModules1_globalAccess", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}

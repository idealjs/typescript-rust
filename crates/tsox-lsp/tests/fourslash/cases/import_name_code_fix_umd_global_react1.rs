use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_umd_global_react1() {
    let content = r#"// @jsx: react
// @allowSyntheticDefaultImports: false
// @module: es2015
// @moduleResolution: bundler
// @Filename: /node_modules/@types/react/index.d.ts
export = React;
export as namespace React;
declare namespace React {
    export class Component { render(): JSX.Element | null; }
}
declare global {
    namespace JSX {
        interface Element {}
    }
}
// @Filename: /a.tsx
[|import { Component } from "react";
export class MyMap extends Component { }
<MyMap></MyMap>;|]"#;
    let mut s = Session::new_for_test("importNameCodeFixUMDGlobalReact1", content);
    fourslash::go_to_file(&mut s, "/a.tsx");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

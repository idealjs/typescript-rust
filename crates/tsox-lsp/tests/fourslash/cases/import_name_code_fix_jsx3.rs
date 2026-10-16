use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_jsx3() {
    let content = r#"// @lib: es5
// @jsx: react
// @module: esnext
// @esModuleInterop: true
// @moduleResolution: bundler
// @Filename: /node_modules/react/index.d.ts
export = React;
export as namespace React;
declare namespace React {
    class Component {}
}
// @Filename: /node_modules/react-native/index.d.ts
import * as React from "react";
export class Text extends React.Component {};
// @Filename: /a.tsx
import React from "react";
<Text></[|Text|]>;"#;
    let mut s = Session::new_for_test("importNameCodeFix_jsx3", content);
    fourslash::go_to_file(&mut s, "/a.tsx");
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

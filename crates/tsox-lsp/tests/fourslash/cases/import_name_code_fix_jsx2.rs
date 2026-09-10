use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn import_name_code_fix_jsx2() {
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
<[|Text|]></Text>;"#;
    let mut s = Session::new_for_test("importNameCodeFix_jsx2", content);
    fourslash::go_to_file(&mut s, "/a.tsx");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

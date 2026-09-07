use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_react_jsx_dev() {
    let content = r#"// @allowSyntheticDefaultImports: true
// @moduleResolution: bundler
// @noUnusedLocals: true
// @target: es2018
// @jsx: react-jsxdev
// @filename: test.tsx
import React from 'react';
export default () => <div></div>
// @filename: node_modules/react/package.json
{
    "name": "react",
    "types": "index.d.ts"
}
// @filename: node_modules/react/index.d.ts
export = React;
declare namespace JSX {
    interface IntrinsicElements { [x: string]: any; }
}
declare namespace React {}
// @filename: node_modules/react/jsx-runtime.d.ts
import './';
// @filename: node_modules/react/jsx-dev-runtime.d.ts
import './';"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "test.tsx");
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}

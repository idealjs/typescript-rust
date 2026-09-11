use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports_react_jsx() {
    let content = r#"// @allowSyntheticDefaultImports: true
// @moduleResolution: bundler
// @noUnusedLocals: true
// @target: es2018
// @jsx: react-jsx
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
    let mut s = Session::new_for_test("organizeImportsReactJsx", content);
    fourslash::go_to_file(&mut s, "test.tsx");
    // TODO: f.VerifyOrganizeImports(t,
}

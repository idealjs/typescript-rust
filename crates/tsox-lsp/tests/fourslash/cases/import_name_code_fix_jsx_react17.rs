use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_jsx_react17() {
    let content = r#"// @jsx: preserve
// @module: commonjs
// @Filename: /node_modules/@types/react/index.d.ts
declare namespace React {
  function createElement(): any;
}
export = React;
export as namespace React;

declare global {
  namespace JSX {
    interface IntrinsicElements {}
    interface IntrinsicAttributes {}
  }  
}
// @Filename: /component.tsx
import "react";
export declare function Component(): any;
// @Filename: /index.tsx
(<Component/**/ />);"#;
    let mut s = Session::new_for_test("importNameCodeFix_jsxReact17", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

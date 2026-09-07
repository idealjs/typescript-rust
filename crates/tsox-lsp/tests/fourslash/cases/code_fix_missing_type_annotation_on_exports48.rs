use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports48() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @moduleResolution: bundler
// @target: es2018
// @jsx: react-jsx
// @filename: node_modules/react/package.json
{
    "name": "react",
    "types": "index.d.ts"
}
// @filename: node_modules/react/index.d.ts
export = React;
declare namespace JSX {
    interface Element extends GlobalJSXElement { }
    interface IntrinsicElements extends GlobalJSXIntrinsicElements { }
}
declare namespace React { }
declare global {
    namespace JSX {
        interface Element { }
        interface IntrinsicElements { [x: string]: any; }
    }
}
interface GlobalJSXElement extends JSX.Element {}
interface GlobalJSXIntrinsicElements extends JSX.IntrinsicElements {}
// @filename: node_modules/react/jsx-runtime.d.ts
import './';
// @filename: node_modules/react/jsx-dev-runtime.d.ts
import './';
// @filename: /a.tsx
export const x = <div aria-label="label text" />;"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/a.tsx");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

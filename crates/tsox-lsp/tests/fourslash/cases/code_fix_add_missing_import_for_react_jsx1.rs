use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_add_missing_import_for_react_jsx1() {
    let content = r#"// @jsx: react-jsx
// @Filename: node_modules/react/index.d.ts
export declare var React: any;
// @Filename: node_modules/react/package.json
{
  "name": "react",
  "types": "./index.d.ts"
}
// @Filename: foo.tsx
 export default function Foo(){
     return <></>;
 }
// @Filename: bar.tsx
 export default function Bar(){
     return <Foo></Foo>;
 }
// @Filename: package.json
{
  "dependencies": {
    "react": "*"
  }
}"#;
    let mut s = Session::new_for_test("codeFixAddMissingImportForReactJsx1", content);
    fourslash::go_to_file(&mut s, "bar.tsx");
    // TODO: f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}

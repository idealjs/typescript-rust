use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn code_fix_add_missing_import_for_react_jsx2() {
    let content = r#"// @jsx: react-jsxdev
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
    let mut s = Session::new_for_test("codeFixAddMissingImportForReactJsx2", content);
    fourslash::go_to_file(&mut s, "bar.tsx");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}

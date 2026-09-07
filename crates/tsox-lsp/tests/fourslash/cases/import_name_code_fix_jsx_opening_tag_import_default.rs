use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_jsx_opening_tag_import_default() {
    let content = r#"// @module: commonjs
// @jsx: react-jsx
// @Filename: /component.tsx
export default function (props: any) {}
// @Filename: /index.tsx
export function Index() {
    return <Component/**/ />;
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

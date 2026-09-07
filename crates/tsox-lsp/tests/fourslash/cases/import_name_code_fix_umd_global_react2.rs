use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_umd_global_react2() {
    let content = r#"// @jsx: react
// @jsxFactory: factory
// @Filename: /factory.ts
export function factory() { return {}; }
declare global {
    namespace JSX {
        interface Element {}
    }
}
// @Filename: /a.tsx
[|<div/>|]"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/a.tsx");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("importNameCodeFixUMDGlobalReact2", content);
    fourslash::go_to_file(&mut s, "/a.tsx");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

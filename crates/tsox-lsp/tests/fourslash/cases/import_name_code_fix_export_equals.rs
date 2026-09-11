use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_export_equals() {
    let content = r#"// @module: commonjs
// @esModuleInterop: false
// @allowSyntheticDefaultImports: false
// @Filename: /a.d.ts
declare function a(): void;
declare namespace a {
    export interface b {}
}
export = a;
// @Filename: /b.ts
a;
let x: b;"#;
    let mut s = Session::new_for_test("importNameCodeFix_exportEquals", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}

use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_umd_global_java_script() {
    let content = r#"// @AllowSyntheticDefaultImports: false
// @Module: commonjs
// @CheckJs: true
// @AllowJs: true
// @Filename: a/f1.js
[|export function test() { };
bar1/*0*/.bar;|]
// @Filename: a/foo.d.ts
export declare function bar(): number;
export as namespace bar1; "#;
    let _s = Session::new_for_test("importNameCodeFixUMDGlobalJavaScript", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

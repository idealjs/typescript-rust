use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_new_import_allow_synthetic_default_imports3() {
    let content = r#"// @AllowSyntheticDefaultImports: false
// @Module: commonjs
// @Filename: a/f1.ts
[|export var x = 0;
bar/*0*/();|]
// @Filename: a/foo.d.ts
declare function bar(): number;
export = bar;
export as namespace bar;"#;
    let _s = Session::new_for_test("importNameCodeFixNewImportAllowSyntheticDefaultImports3", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

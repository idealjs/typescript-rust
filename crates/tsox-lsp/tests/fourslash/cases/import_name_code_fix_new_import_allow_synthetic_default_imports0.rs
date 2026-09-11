use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_new_import_allow_synthetic_default_imports0() {
    let content = r#"// @AllowSyntheticDefaultImports: true
// @Filename: a/f1.ts
[|export var x = 0;
bar/*0*/();|]
// @Filename: a/foo.d.ts
declare function bar(): number;
export = bar;
export as namespace bar;"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportAllowSyntheticDefaultImports0", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

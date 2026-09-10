use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_allow_synthetic_default_imports5() {
    let content = r#"// @AllowSyntheticDefaultImports: false
// @Module: umd
// @Filename: a/f1.ts
[|export var x = 0;
bar/*0*/();|]
// @Filename: a/foo.d.ts
declare function bar(): number;
export = bar;
export as namespace bar;"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportAllowSyntheticDefaultImports5", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

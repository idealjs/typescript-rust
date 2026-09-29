use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_umd_global0() {
    let content = r#"// @AllowSyntheticDefaultImports: false
// @Module: es2015
// @Filename: a/f1.ts
[|export function test() { };
bar1/*0*/.bar;|]
// @Filename: a/foo.d.ts
export declare function bar(): number;
export as namespace bar1; "#;
    let _s = Session::new_for_test("importNameCodeFixUMDGlobal0", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

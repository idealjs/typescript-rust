use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_default_export5() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /node_modules/hooks/useFoo.ts
declare const _default: () => void;
export default _default;
// @Filename: /test.ts
[|useFoo|];"#;
    let mut s = Session::new_for_test("importNameCodeFixDefaultExport5", content);
    // TODO: f.Configure(t, lsutil.UserPreferences{AutoImportEntrypointDirectorySearch: core.TSTrue})
    fourslash::go_to_file(&mut s, "/test.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

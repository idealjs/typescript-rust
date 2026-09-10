use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Configure"]
#[test]
fn import_name_code_fix_default_export5() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /node_modules/hooks/useFoo.ts
declare const _default: () => void;
export default _default;
// @Filename: /test.ts
[|useFoo|];"#;
    let mut s = Session::new_for_test("importNameCodeFixDefaultExport5", content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{AutoImportEntrypointDirectorySearch: core.TSTrue})
    fourslash::go_to_file(&mut s, "/test.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

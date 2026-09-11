use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_new_import_index_not_for_classic_resolution() {
    let content = r#"// @moduleResolution: classic
// @Filename: /a/index.ts
export const foo = 0;
// @Filename: /node_modules/x/index.d.ts
export const bar = 0;
// @Filename: /b.ts
[|foo;|]
// @Filename: /c.ts
[|bar;|]"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportIndex_notForClassicResolution", content);
    fourslash::go_to_file(&mut s, "/a/index.ts");
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/c.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
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
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/a/index.ts");
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/c.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

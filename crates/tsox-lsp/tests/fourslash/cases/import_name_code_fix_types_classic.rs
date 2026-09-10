use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn import_name_code_fix_types_classic() {
    let content = r#"// @moduleResolution: classic
// @Filename: /node_modules/@types/foo/index.d.ts
export const xyz: number;
// @Filename: /node_modules/bar/index.d.ts
export const qrs: number;
// @Filename: /a.ts
xyz;
qrs;"#;
    let mut s = Session::new_for_test("importNameCodeFix_types_classic", content);
    fourslash::go_to_file(&mut s, "/a.ts");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}

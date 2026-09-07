use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn import_name_code_fix_add_all_missing_imports() {
    let content = r#"// @Filename: /a.ts
export const a: number;
// @Filename: /b.ts
export const b: number;
// @Filename: /c.ts
export const c: number;
// @Filename: /main.ts
a;
b;
c;"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/main.ts");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}

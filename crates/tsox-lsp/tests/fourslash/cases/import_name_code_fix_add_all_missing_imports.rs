use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("importNameCodeFix_add_all_missing_imports", content);
    fourslash::go_to_file(&mut s, "/main.ts");
    // TODO: f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}

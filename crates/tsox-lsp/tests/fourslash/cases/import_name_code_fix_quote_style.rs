use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_quote_style() {
    let content = r#"// @Filename: /a.ts
export const foo: number;
// @Filename: /b.ts
[|foo;|]"#;
    let mut s = Session::new_for_test("importNameCodeFix_quoteStyle", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

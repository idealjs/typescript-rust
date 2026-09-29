use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_re_export() {
    let content = r#"// @Filename: /a.ts
export default function foo(): void {}
// @Filename: /b.ts
export { default } from "./a";
// @Filename: /user.ts
[|foo;|]"#;
    let mut s = Session::new_for_test("importNameCodeFix_reExport", content);
    fourslash::go_to_file(&mut s, "/user.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

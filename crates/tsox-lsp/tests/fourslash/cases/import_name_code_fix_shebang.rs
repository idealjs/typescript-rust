use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_shebang() {
    let content = r#"// @Filename: /a.ts
export const foo = 0;
// @Filename: /b.ts
[|#!/usr/bin/env node
foo/**/|]"#;
    let mut s = Session::new_for_test("importNameCodeFixShebang", content);
    fourslash::go_to_file(&mut s, "/a.ts");
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

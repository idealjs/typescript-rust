use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_shebang() {
    let content = r#"// @Filename: /a.ts
export const foo = 0;
// @Filename: /b.ts
[|#!/usr/bin/env node
foo/**/|]"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/a.ts");
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

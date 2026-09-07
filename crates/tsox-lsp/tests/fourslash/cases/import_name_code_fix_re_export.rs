use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyRangeAfterCodeFix"]
#[test]
fn import_name_code_fix_re_export() {
    let content = r#"// @Filename: /a.ts
export const x = 0";
// @Filename: /b.ts
[|export { x } from "./a";
x;|]"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `import { x } from "./a";
}

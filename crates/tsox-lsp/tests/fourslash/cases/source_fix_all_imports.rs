use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // Test per-fixId fix-all via VerifyCodeFixAll (quickfix pat"]
#[test]
fn source_fix_all_imports() {
    let content = r#"// @Filename: /a.ts
export const a: number = 1;
// @Filename: /b.ts
export const b: number = 2;
// @Filename: /main.ts
a;
b;"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/main.ts");
    // TODO: // Test per-fixId fix-all via VerifyCodeFixAll (quickfix path)
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}

#[ignore = "generator: // Test source.fixAll code action directly (on-save path)"]
#[test]
fn source_fix_all_code_action() {
    let content = r#"// @Filename: /a.ts
export const a: number = 1;
// @Filename: /b.ts
export const b: number = 2;
// @Filename: /main.ts
a;
b;"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/main.ts");
    // TODO: // Test source.fixAll code action directly (on-save path)
    fourslash::unsupported("VerifySourceFixAll"); // f.VerifySourceFixAll(t, `import { a } from "./a";
}

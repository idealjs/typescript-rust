use tsox_lsp::fourslash::{self, Session};


#[test]
fn source_fix_all_imports() {
    let content = r#"// @Filename: /a.ts
export const a: number = 1;
// @Filename: /b.ts
export const b: number = 2;
// @Filename: /main.ts
a;
b;"#;
    let mut s = Session::new_for_test("sourceFixAllImports", content);
    fourslash::go_to_file(&mut s, "/main.ts");
    // TODO: // Test per-fixId fix-all via VerifyCodeFixAll (quickfix path)
    // TODO: f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}

#[test]
fn source_fix_all_code_action() {
    let content = r#"// @Filename: /a.ts
export const a: number = 1;
// @Filename: /b.ts
export const b: number = 2;
// @Filename: /main.ts
a;
b;"#;
    let mut s = Session::new_for_test("sourceFixAllCodeAction", content);
    fourslash::go_to_file(&mut s, "/main.ts");
    // TODO: // Test source.fixAll code action directly (on-save path)
    // TODO: f.VerifySourceFixAll(t, `import { a } from "./a";
}

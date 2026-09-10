use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_order() {
    let content = r#"// @Filename: /a.ts
export const foo: number;
// @Filename: /b.ts
export const foo: number;
export const bar: number;
// @Filename: /c.ts
[|import { bar } from "./b";
foo;|]"#;
    let mut s = Session::new_for_test("importNameCodeFix_order", content);
    fourslash::go_to_file(&mut s, "/c.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

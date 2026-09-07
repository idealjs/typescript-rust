use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_type_used_as_value() {
    let content = r#"// @Filename: /a.ts
export class ReadonlyArray<T> {}
// @Filename: /b.ts
[|new ReadonlyArray<string>();|]"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

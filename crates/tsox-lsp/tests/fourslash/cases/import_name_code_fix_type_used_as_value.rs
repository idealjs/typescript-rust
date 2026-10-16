use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_type_used_as_value() {
    let content = r#"// @Filename: /a.ts
export class ReadonlyArray<T> {}
// @Filename: /b.ts
[|new ReadonlyArray<string>();|]"#;
    let mut s = Session::new_for_test("importNameCodeFix_typeUsedAsValue", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

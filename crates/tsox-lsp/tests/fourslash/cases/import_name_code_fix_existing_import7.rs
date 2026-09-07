use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_existing_import7() {
    let content = r#"import [|{ v1 }|] from "../other_dir/module";
f1/*0*/();
// @Filename: ../other_dir/module.ts
export var v1 = 5;
export function f1();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

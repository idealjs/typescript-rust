use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_default_export2() {
    let content = r#"// @Filename: /lib.ts
class Base { }
export default Base;
// @Filename: /test.ts
[|class Derived extends Base { }|]"#;
    let mut s = Session::new_for_test("importNameCodeFixDefaultExport2", content);
    fourslash::go_to_file(&mut s, "/test.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

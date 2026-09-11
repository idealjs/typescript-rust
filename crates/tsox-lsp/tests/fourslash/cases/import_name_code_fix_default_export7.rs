use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_default_export7() {
    let content = r#"// @lib: dom
// @Filename: foo.ts
export default globalThis.localStorage;
// @Filename: index.ts
foo/**/"#;
    let mut s = Session::new_for_test("importNameCodeFixDefaultExport7", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{}, nil /*preferences*/)
}

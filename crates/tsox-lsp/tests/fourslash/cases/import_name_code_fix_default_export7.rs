use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_default_export7() {
    let content = r#"// @lib: dom
// @Filename: foo.ts
export default globalThis.localStorage;
// @Filename: index.ts
foo/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{}, nil /*preferences*/)
}

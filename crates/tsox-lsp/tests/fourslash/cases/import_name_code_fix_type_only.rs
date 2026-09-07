use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_type_only() {
    let content = r#"// @module: esnext
// @verbatimModuleSyntax: true
// @Filename: types.ts
export class A {}
// @Filename: index.ts
const a: /**/A"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

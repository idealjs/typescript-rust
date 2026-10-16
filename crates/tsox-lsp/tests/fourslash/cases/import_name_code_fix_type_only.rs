use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_type_only() {
    let content = r#"// @module: esnext
// @verbatimModuleSyntax: true
// @Filename: types.ts
export class A {}
// @Filename: index.ts
const a: /**/A"#;
    let mut s = Session::new_for_test("importNameCodeFix_typeOnly", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

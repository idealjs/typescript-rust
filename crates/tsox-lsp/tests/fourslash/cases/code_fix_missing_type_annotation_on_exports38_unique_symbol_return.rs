use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports38_unique_symbol_return() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
// @Filename: /code.ts
const u: unique symbol = Symbol();
export const fn = () => ({ u } as const);"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports38_unique_symbol_return", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

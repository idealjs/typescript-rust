use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports59_drops_unneeded_after_unknown() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true

export interface Foo<S = string, T = unknown, U = number> {}
export function g(x: Foo<number, unknown, number>) { return x; }
"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports59_drops_unneeded_after_unknown", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports60_drops_unneeded_non_trailing_unknown() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true

export interface Foo<S = string, T = unknown> {}
export function f(x: Foo<string, unknown>) { return x; }
"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports60_drops_unneeded_non_trailing_unknown", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports_jsx_whitespace_text() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @module: preserve
// @Filename: /test.tsx
export const /**/elem = <div>
    <span />
</div>;"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports_jsxWhitespaceText", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCodeFixAvailable(t, nil)
}

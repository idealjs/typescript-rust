use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports_type_predicate1() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @filename: index.ts
export function isString(value: unknown) {
  return typeof value === "string";
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

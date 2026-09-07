use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports37_array_spread() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @Filename: /code.ts
const Start = [
  'A',
  'B',
] as const;

const End = [
  "Y",
  "Z"
] as const;
export const All_Part1 = {};
function getPart() {
  return ["Z"]
}

export const All = [
  1,
  ...Start,
  1,
  ...getPart(),
  ...End,
  1,
] as const;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

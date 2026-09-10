use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports34_object_spread() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @Filename: /code.ts
const Start = {
  A: 'A',
  B: 'B',
} as const;

const End = {
  Y: "Y",
  Z: "Z"
} as const;
export const All_Part1 = {};
function getPart() {
  return { M: "Z"}
}

export const All = {
  x: 1,
  ...Start,
  y: 1,
  ...getPart(),
  ...End,
  z: 1,
};"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports34_object_spread", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

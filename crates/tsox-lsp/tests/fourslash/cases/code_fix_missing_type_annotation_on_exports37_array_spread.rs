use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports37_array_spread", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

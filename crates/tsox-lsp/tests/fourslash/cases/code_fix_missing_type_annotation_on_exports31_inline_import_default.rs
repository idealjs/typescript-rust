use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports31_inline_import_default() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @Filename: /person-code.ts
export type Person = { x: string; }
export function getPerson() : Person {
  return null!
}
// @Filename: /code.ts
import { getPerson } from "./person-code";
export default {
  person: getPerson()
};"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports31_inline_import_default", content);
    fourslash::go_to_file(&mut s, "/code.ts");
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Extract default export to variable", "Add satisfies and an inl
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

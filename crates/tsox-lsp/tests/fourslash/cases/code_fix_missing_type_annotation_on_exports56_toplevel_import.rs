use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports56_toplevel_import() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @Filename: /person-code.ts
export interface Person { x: string; }
export function getPerson() : Person {
  return null!
}
// @Filename: /code.ts
import { getPerson } from "./person-code";
export function wrapPerson() {
  return { person: getPerson() }
};"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports56_toplevel_import", content);
    fourslash::go_to_file(&mut s, "/code.ts");
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports30_inline_import() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @Filename: /person-code.ts
export type Person = { x: string; }
export function getPerson() : Person {
  return null!
}
// @Filename: /code.ts
import { getPerson } from "./person-code";
export const exp = {
  person: getPerson()
};"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/code.ts");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}

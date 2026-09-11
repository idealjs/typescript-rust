use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_type_import3() {
    let content = r#"// @verbatimModuleSyntax: true
// @target: esnext
// @Filename: /foo.ts
export const A = 1;
export type B = { x: number };
export type C = 1;
export class D = { y: string };
// @Filename: /test.ts
import { A, type B, type C } from './foo';
const b: B | C;
console.log(A, D/**/);"#;
    let mut s = Session::new_for_test("autoImportTypeImport3", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

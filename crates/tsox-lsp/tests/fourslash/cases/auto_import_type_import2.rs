use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn auto_import_type_import2() {
    let content = r#"// @verbatimModuleSyntax: true
// @target: esnext
// @Filename: /foo.ts
export const A = 1;
export type B = { x: number };
export type C = 1;
export class D = { y: string };
// @Filename: /test.ts
import { A, type C, D } from './foo';
const b: B/**/ | C;
console.log(A, D);"#;
    let mut s = Session::new_for_test("autoImportTypeImport2", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

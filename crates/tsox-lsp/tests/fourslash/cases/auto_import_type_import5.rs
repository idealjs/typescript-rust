use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn auto_import_type_import5() {
    let content = r#"// @verbatimModuleSyntax: true
// @target: esnext
// @Filename: /exports1.ts
export const a = 0;
export const A = 1;
export const b = 2;
export const B = 3;
export const c = 4;
export const C = 5;
export type x = 6;
export const X = 7;
export type y = 8
export const Y = 9;
export const Z = 10;
// @Filename: /exports2.ts
export const d = 0;
export const D = 1;
export const e = 2;
export const E = 3;
// @Filename: /index0.ts
import { type X, type Y, type Z } from "./exports1";
const foo: x/*0*/;
const bar: y;
// @Filename: /index1.ts
import { A, B, type X, type Y, type Z } from "./exports1";
const foo: x/*1*/;
const bar: y;"#;
    let mut s = Session::new_for_test("autoImportTypeImport5", content);
    fourslash::go_to_marker(&mut s, "0");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

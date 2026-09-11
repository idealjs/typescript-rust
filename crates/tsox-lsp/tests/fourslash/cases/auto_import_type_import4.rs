use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_type_import4() {
    // TODO: t.Skip("Known failing fourslash test")
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
export const Y = 8;
export const Z = 9;
// @Filename: /exports2.ts
export const d = 0;
export const D = 1;
export const e = 2;
export const E = 3;
// @Filename: /index0.ts
import { A, B, C } from "./exports1";
a/*0*//*0a*/;
b;
// @Filename: /index1.ts
import { A, B, C, type Y, type Z } from "./exports1";
a/*1*//*1a*//*1b*//*1c*/;
b;
// @Filename: /index2.ts
import { A, a, B, b, type Y, type Z } from "./exports1";
import { E } from "./exports2";
d/*2*//*2a*//*2b*//*2c*/"#;
    let mut s = Session::new_for_test("autoImportTypeImport4", content);
    fourslash::go_to_marker(&mut s, "0");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "0a");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "1a");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "1b");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "1c");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "2a");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "2b");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "2c");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

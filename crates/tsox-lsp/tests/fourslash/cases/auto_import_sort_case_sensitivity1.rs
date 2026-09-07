use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_sort_case_sensitivity1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /exports1.ts
export const a = 0;
export const A = 1;
export const b = 2;
export const B = 3;
export const c = 4;
export const C = 5;
// @Filename: /exports2.ts
export const d = 0;
export const D = 1;
export const e = 2;
export const E = 3;
// @Filename: /index0.ts
import { A, B, C } from "./exports1";
a/*0*/
// @Filename: /index1.ts
import { A, a, B, b } from "./exports1";
import { E } from "./exports2";
d/*1*/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "0");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

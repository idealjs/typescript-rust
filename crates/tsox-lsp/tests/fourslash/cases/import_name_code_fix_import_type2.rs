use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_import_type2() {
    let content = r#"// @verbatimModuleSyntax: true
// @module: es2015
// @Filename: /exports1.ts
export default interface SomeType {}
export interface OtherType {}
export interface OtherOtherType {}
export const someValue = 0;
// @Filename: /a.ts
import type SomeType from "./exports1.js";
someValue/*a*/
// @Filename: /b.ts
import { someValue } from "./exports1.js";
const b: SomeType/*b*/ = someValue;
// @Filename: /c.ts
import type SomeType from "./exports1.js";
const x: OtherType/*c*/
// @Filename: /d.ts
import type { OtherType } from "./exports1.js";
const x: OtherOtherType/*d*/"#;
    let mut s = Session::new_for_test("importNameCodeFix_importType2", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "b");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "c");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "d");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_import_type1() {
    let content = r#"// @verbatimModuleSyntax: true
// @module: es2015
// @Filename: /exports.ts
export default someValue = 0;
export function Component() {}
export interface ComponentProps {}
// @Filename: /a.ts
import { Component } from "./exports.js";
interface MoreProps extends /*a*/ComponentProps {}
// @Filename: /b.ts
import someValue from "./exports.js";
interface MoreProps extends /*b*/ComponentProps {}"#;
    let mut s = Session::new_for_test("importNameCodeFix_importType1", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "b");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_js_doc_import1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @verbatimModuleSyntax: true
// @target: esnext
// @allowJs: true
// @checkJs: true
// @Filename: /foo.ts
 export const A = 1;
 export type B = { x: number };
 export type C = 1;
 export class D { y: string }
// @Filename: /test.js
/**
 * @import { A, D, C } from "./foo"
 */

/**
 * @param { typeof A } a
 * @param { B/**/ | C } b
 * @param { C } c
 * @param { D } d
 */
export function f(a, b, c, d) { }"#;
    let mut s = Session::new_for_test("autoImportJsDocImport1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

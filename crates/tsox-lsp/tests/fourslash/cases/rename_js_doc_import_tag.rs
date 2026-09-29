use tsox_lsp::fourslash::Session;


#[test]
fn rename_js_doc_import_tag() {
    let content = r#"// @allowJS: true
// @checkJs: true
// @Filename: /b.ts
export interface A { }
// @Filename: /a.js
/**
 * @import { A } from "./b";
 */

/**
 * @param { [|A/**/|] } a
 */
function f(a) {}"#;
    let _s = Session::new_for_test("renameJsDocImportTag", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}

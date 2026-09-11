use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_js_doc_import_tag() {
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
    let mut s = Session::new_for_test("findAllRefsJsDocImportTag", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}

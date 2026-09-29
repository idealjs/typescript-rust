use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_js_doc_import_tag5() {
    let content = r#"// @allowJS: true
// @checkJs: true
// @Filename: /b.ts
export interface /*2*/A { }
// @Filename: /a.js
/**
 * @import { A } from "./b";
 */

/**
 * @param { [|A/*1*/|] } a
 */
function f(a) {}"#;
    let _s = Session::new_for_test("goToDefinitionJsDocImportTag5", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}

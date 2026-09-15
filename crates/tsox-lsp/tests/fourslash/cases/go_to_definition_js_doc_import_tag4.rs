use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_js_doc_import_tag4() {
    let content = r#"// @allowJS: true
// @checkJs: true
// @Filename: /b.ts
export interface /*2*/A { }
// @Filename: /a.js
/**
 * @import { [|A/*1*/|] } from "./b";
 */"#;
    let _s = Session::new_for_test("goToDefinitionJsDocImportTag4", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}

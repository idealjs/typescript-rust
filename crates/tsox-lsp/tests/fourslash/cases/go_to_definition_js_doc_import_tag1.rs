use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_js_doc_import_tag1() {
    let content = r#"// @allowJS: true
// @checkJs: true
// @Filename: /b.ts
/*2*/export interface A { }
// @Filename: /a.js
/**
 * @import { A } from      [|"./b/*1*/"|]
 */"#;
    let mut s = Session::new_for_test("goToDefinitionJsDocImportTag1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}

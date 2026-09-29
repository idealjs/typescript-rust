use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_js_doc_import_tag2() {
    let content = r#"// @allowJS: true
// @checkJs: true
// @Filename: /b.ts
/*2*/export interface A { }
// @Filename: /a.js
/**
 * @import { A } [|from/*1*/|]       "./b"
 */"#;
    let _s = Session::new_for_test("goToDefinitionJsDocImportTag2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}

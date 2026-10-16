use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_js_doc_import_tag_attributes_error_module_specifier1() {
    let content = r#"// @strict: true
// @checkJs: true
// @allowJs: true
// @filename: global.d.ts
interface ImportAttributes { 
  type: "json";
}
// @filename: index.js
/** @import * as ns from () with { type: "/**/" } */"#;
    let mut s = Session::new_for_test("completionsJSDocImportTagAttributesErrorModuleSpecifier1", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["json"]);
}

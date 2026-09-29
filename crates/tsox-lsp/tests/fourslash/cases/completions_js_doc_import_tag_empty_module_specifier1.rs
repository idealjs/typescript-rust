use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_js_doc_import_tag_empty_module_specifier1() {
    let content = r#"// @strict: true
// @checkJs: true
// @allowJs: true
// @moduleResolution: nodenext
// @filename: node_modules/pkg/index.d.ts
export type MyUnion = string | number;
// @filename: index.js
/** @import { MyUnion } from "/**/" */"#;
    let mut s = Session::new_for_test("completionsJSDocImportTagEmptyModuleSpecifier1", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["pkg"]);
}

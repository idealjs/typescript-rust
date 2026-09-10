use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_declaration_attributes_empty_module_specifier1() {
    let content = r#"// @strict: true
// @filename: global.d.ts
interface ImportAttributes { 
  type: "json";
}
// @filename: index.ts
import * as ns from "" with { type: "/**/" };"#;
    let mut s = Session::new_for_test("completionsImportDeclarationAttributesEmptyModuleSpecifier1", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["json"]);
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_import_declaration_attributes_error_module_specifier1() {
    let content = r#"// @strict: true
// @filename: global.d.ts
interface ImportAttributes { 
  type: "json";
}
// @filename: index.ts
import * as ns from () with { type: "/**/" };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

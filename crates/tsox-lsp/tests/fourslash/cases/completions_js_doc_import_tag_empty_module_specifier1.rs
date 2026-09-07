use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_statement_completions_es_module_interop1() {
    let content = r#"// @esModuleInterop: false
// @allowSyntheticDefaultImports: false
// @module: commonjs
// @Filename: /mod.ts
const foo = 0;
export = foo;
// @Filename: /importExportEquals.ts
[|import f/**/|]"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

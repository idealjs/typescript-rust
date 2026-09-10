use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn import_statement_completions_js() {
    let content = r#"// @allowJs: true
// @target: es2020
// @checkJs: true
// @module: commonjs
// @noEmit: true
// @allowSyntheticDefaultImports: true
// @Filename: /node_modules/react/index.d.ts
declare namespace React {
   export class Component {}
}
export = React;
// @Filename: /test.js
[|import R/**/|]"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

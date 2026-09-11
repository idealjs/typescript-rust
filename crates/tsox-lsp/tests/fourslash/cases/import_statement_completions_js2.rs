use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_statement_completions_js2() {
    let content = r#"// @allowJs: true
// @target: es2020
// @checkJs: true
// @module: commonjs
// @esModuleInterop: false
// @allowSyntheticDefaultImports: false
// @noEmit: true
// @Filename: /node_modules/react/index.d.ts
declare namespace React {
   export class Component {}
}
export = React;
// @Filename: /test.js
[|import R/**/|]"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

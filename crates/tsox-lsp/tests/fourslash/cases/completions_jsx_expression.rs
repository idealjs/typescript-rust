use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_jsx_expression() {
    let content = r#"// @Filename: /a.tsx
// @jsx: react
declare namespace JSX {
    interface IntrinsicElements {
        div: { a: string, b: string }
    }
}
const value = "test";
<div a={v/**/} />"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

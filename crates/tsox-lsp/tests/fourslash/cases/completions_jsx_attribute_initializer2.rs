use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_jsx_attribute_initializer2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /a.tsx
declare namespace JSX {
    interface IntrinsicElements {
        div: { a: string, b: string }
    }
}
const foo = 0;
<div x=[|f/*0*/|] />;

<div a="1" b/*1*/ />
<div a /*2*/ />"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_jsx_attribute2() {
    let content = r#"// @jsx: preserve
// @Filename: /a.tsx
declare namespace JSX {
    interface Element {}
    interface IntrinsicElements {
        div: {
            /** Doc */
            foo: boolean;
            bar: string;
            "aria-foo": boolean;
        }
    }
}

<div foo /*1*/></div>;
<div foo={true} /*2*/></div>;
<div bar="test" /*3*/></div>;
<div aria-foo /*4*/></div>;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}

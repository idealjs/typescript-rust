use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn jsx_tag_name_completion_with_existing_jsx_initializer() {
    let content = r#"// @filename: /foo.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        foo: {
            className: string;
        }
    }
}
<foo cl/**/={""} />"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

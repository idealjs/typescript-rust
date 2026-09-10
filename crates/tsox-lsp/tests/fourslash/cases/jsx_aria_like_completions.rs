use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn jsx_aria_like_completions() {
    let content = r#"//@Filename: file.tsx
declare var React: any;
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        div: { "aria-whatever"?: string  }
    }
    interface ElementAttributesProperty { props: any }
}
const a = <div {...{}} /*1*/></div>;"#;
    let mut s = Session::new_for_test("jsxAriaLikeCompletions", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

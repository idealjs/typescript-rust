use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_in_jsx_tag() {
    let content = r#"// @jsx: preserve
// @Filename: /a.tsx
declare namespace JSX {
    interface Element {}
    interface IntrinsicElements {
        div: {
            /** Doc */
            foo: string
            /** Label docs */
            "aria-label": string
        }
    }
}
class Foo {
    render() {
        <div /*1*/ ></div>;
        <div  /*2*/ />
    }
}"#;
    let mut s = Session::new_for_test("completionsInJsxTag", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_in_jsx_namespaced_intrinsic_tag() {
    let content = r#"// @jsx: react
// @Filename: /a.tsx
declare const React: any;
declare namespace JSX {
    interface Element {}
    interface IntrinsicElements {
        /** Element docs */
        "foo:bar": {
            /** Foo docs */
            foo: boolean
            /** Bar docs */
            bar: string
        }
    }
}
<foo:bar /*1*/ />
<foo:bar  /*2*/></foo:bar>"#;
    let mut s = Session::new_for_test("completionsInJsxNamespacedIntrinsicTag", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}

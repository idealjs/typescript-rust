use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_jsx_attribute_initializer2() {
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
    let mut s = Session::new_for_test("completionsJsxAttributeInitializer2", content);
    fourslash::go_to_marker(&mut s, "0");
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}

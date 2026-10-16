use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsx_tag_name_completion_under_element_unclosed() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface IntrinsicElements {
        button: any;
        div: any;
    }
}
function fn() {
    return <>
        <butto/*1*/
    </>;
}
function fn2() {
    return <>
        preceding junk <butto/*2*/
    </>;
}
function fn3() {
    return <>
        <butto/*3*/ style=""
    </>;
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}

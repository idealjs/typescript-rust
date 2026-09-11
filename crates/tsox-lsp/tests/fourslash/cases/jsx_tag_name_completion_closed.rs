use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsx_tag_name_completion_closed() {
    let content = r#"//@Filename: file.tsx
interface NestedInterface {
    Foo: NestedInterface;
    (props: {}): any;
}

declare const Foo: NestedInterface;

function fn1() {
    return <Foo>
        </*1*/ />
    </Foo>
}
function fn2() {
    return <Foo>
        <Fo/*2*/ />
    </Foo>
}
function fn3() {
    return <Foo>
        <Foo./*3*/ />
    </Foo>
}
function fn4() {
    return <Foo>
        <Foo.F/*4*/ />
    </Foo>
}
function fn5() {
    return <Foo>
        <Foo.Foo./*5*/ />
    </Foo>
}
function fn6() {
    return <Foo>
        <Foo.Foo.F/*6*/ />
    </Foo>
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "5");
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "6");
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
}

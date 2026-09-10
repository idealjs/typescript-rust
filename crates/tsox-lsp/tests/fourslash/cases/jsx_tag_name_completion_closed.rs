use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
}

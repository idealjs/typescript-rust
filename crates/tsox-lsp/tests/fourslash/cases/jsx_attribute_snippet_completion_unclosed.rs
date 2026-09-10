use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn jsx_attribute_snippet_completion_unclosed() {
    let content = r#"// @strict: false
//@Filename: file.tsx
interface NestedInterface {
    Foo: NestedInterface;
    (props: {className?: string}): any;
}

declare const Foo: NestedInterface;

function fn1() {
    return <Foo>
        <Foo /*1*/
    </Foo>
}
function fn2() {
    return <Foo>
        <Foo.Foo /*2*/
    </Foo>
}
function fn3() {
    return <Foo>
        <Foo.Foo [|cla/*3*/|]
    </Foo>
}
function fn4() {
    return <Foo>
        <Foo.Foo [|cla/*4*/|] something
    </Foo>
}
function fn5() {
    return <Foo>
        <Foo.Foo something /*5*/
    </Foo>
}
function fn6() {
    return <Foo>
        <Foo.Foo something [|cla/*6*/|]
    </Foo>
}
function fn7() {
    return <Foo /*7*/
}
function fn8() {
    return <Foo [|cla/*8*/|]
}
function fn9() {
    return <Foo [|cla/*9*/|] something
}
function fn10() {
    return <Foo something /*10*/
}
function fn11() {
    return <Foo something [|cla/*11*/|]
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "8", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "9", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "10", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "11", &fourslash.CompletionsExpectedList{
}

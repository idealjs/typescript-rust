use tsox_lsp::fourslash::{self, Session};


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
    fourslash::go_to_marker(&mut s, "7");
    // TODO: f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "8");
    // TODO: f.VerifyCompletions(t, "8", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "9");
    // TODO: f.VerifyCompletions(t, "9", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "10");
    // TODO: f.VerifyCompletions(t, "10", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "11");
    // TODO: f.VerifyCompletions(t, "11", &fourslash.CompletionsExpectedList{
}

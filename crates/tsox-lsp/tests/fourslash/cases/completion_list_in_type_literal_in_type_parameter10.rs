use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_literal_in_type_parameter10() {
    let content = r#"interface Foo {
    one: string;
    two: number;
}
interface Bar {
    three: boolean;
    four: {
        five: unknown;
    };
}

function a<T extends Foo>() {}
a<{/*0*/}>();

var b = () => <T extends Foo>() => {};
b()<{/*1*/}>();

declare function c<T extends Foo>(): void
declare function c<T extends Bar>(): void
c<{/*2*/}>();

function d<T extends Foo, U extends Bar>() {}
d<{/*3*/}, {/*4*/}>();
d<Foo, { four: {/*5*/} }>();

(<T extends Foo>() => {})<{/*6*/}>();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
}

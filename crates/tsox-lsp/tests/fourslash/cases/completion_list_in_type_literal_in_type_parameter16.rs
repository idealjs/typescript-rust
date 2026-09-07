use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_literal_in_type_parameter16() {
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

(<T extends Foo>() => {})<{/*0*/}>;

(class <T extends Foo>{})<{/*1*/}>;

declare const a: {
    new <T extends Foo>(): {};
    <T extends Bar>(): {};
}
a<{/*2*/}>;

declare const b: {
    new <T extends { one: true }>(): {};
    <T extends { one: false }>(): {};
}
b<{/*3*/}>;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}

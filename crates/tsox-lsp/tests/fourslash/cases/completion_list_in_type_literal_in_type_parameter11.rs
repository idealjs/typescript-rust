use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_literal_in_type_parameter11() {
    let content = r#"interface Foo {
    one: string;
    two: number;
}
interface Bar {
    three: boolean;
    four: symbol;
}

class A<T extends Foo> {}
new A<{/*0*/}>();

class B<T extends Foo, U extends Bar> {}
new B<{/*1*/}, {/*2*/}>();

declare const C: {
   new <T extends Foo>(): unknown
   new <T extends Bar>(): unknown
}
new C<{/*3*/}>()

new (class <T extends Foo> {})<{/*4*/}>();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}

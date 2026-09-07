use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_literal_in_type_parameter19() {
    let content = r#"class Foo<T extends 'one' | 'two'> {}
function foo<T extends 'one' | 'two'>() {}
declare function tag<T extends 'one' | 'two'>(x: TemplateStringsArray): void;
declare function decorator<T extends 'one' | 'two'>(...args: unknown[]): never

type A = Foo<'/*0*/'>;
new Foo<'/*1*/'>();
foo<'/*2*/'>();
foo<'/*3*/'>;
Foo<'/*4*/'>;
tag<'/*5*/'>` + "`" + `` + "`" + `;
class { @decorator<'/*6*/'>; method() {} }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_literal_in_type_parameter18() {
    let content = r#"class Foo<T extends { x: 'one' | 'two' }> {}
function foo<T extends { x: 'one' | 'two' }>() {}
declare function tag<T extends { x: 'one' | 'two' }>(x: TemplateStringsArray): void;
declare function decorator<T extends { x: 'one' | 'two' }>(...args: unknown[]): never

type A = Foo<{ x: '/*0*/' }>;
new Foo<{ x: '/*1*/' }>();
foo<{ x: '/*2*/' }>();
foo<{ x: '/*3*/' }>;
Foo<{ x: '/*4*/' }>;
tag<{ x: '/*5*/' }>` + "`" + `` + "`" + `;
class { @decorator<{ x: '/*6*/' }>; method() {} }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
}

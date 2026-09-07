use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_literal_in_type_parameter20() {
    let content = r#"// @jsx: preserve
// @filename: a.tsx
const Component1 = <T extends { x: 'one' | 'two' }>() => <></>;
const Component2 = <T extends 'one' | 'two'>() => <></>;

<Component1<{ x: '/*0*/' }>></Component>;
<Component1<{ x: '/*1*/' }>/>;
<Component2<'/*2*/'>></Component>;
<Component2<'/*3*/'>/>;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}

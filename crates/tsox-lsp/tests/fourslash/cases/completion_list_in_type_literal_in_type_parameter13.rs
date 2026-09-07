use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_literal_in_type_parameter13() {
    let content = r#"// @jsx: preserve
// @filename: a.tsx
interface Foo {
    one: string;
    two: number;
}

const Component = <T extends Foo>() => <></>;

<Component<{/*0*/}>></Component>;
<Component<{/*1*/}>/>;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

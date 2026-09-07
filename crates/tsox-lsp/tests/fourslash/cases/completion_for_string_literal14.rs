use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal14() {
    let content = r#"interface Foo {
    a: string;
    b: boolean;
    c: number;
}
type Bar = Record<keyof Foo, any>["[|/**/|]"];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

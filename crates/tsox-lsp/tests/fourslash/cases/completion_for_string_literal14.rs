use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal14() {
    let content = r#"interface Foo {
    a: string;
    b: boolean;
    c: number;
}
type Bar = Record<keyof Foo, any>["[|/**/|]"];"#;
    let mut s = Session::new_for_test("completionForStringLiteral14", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

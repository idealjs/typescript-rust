use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_literal_in_type_parameter14() {
    let content = r#"interface Foo {
   one: string;
   two: number;
}
declare function f<T extends Foo>(x: TemplateStringsArray): void;
f<{/*0*/}>` + "`" + `` + "`" + `;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
}

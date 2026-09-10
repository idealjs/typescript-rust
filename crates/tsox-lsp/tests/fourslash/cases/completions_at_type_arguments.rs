use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_at_type_arguments() {
    let content = r#"interface I {
    a: string;
    b: number;
}
type T1 = Pick<I, "/*1*/">;
interface T2 extends Pick<I, "/*2*/"> {}"#;
    let mut s = Session::new_for_test("completionsAtTypeArguments", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}

use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_combine_overloads_rest_parameter() {
    let content = r#"interface A { a: number }
interface B { b: number }
interface C { c: number }
declare function f(a: A): void;
declare function f(...bs: B[]): void;
declare function f(...cs: C[]): void;
f({ /*1*/ });
f({ a: 1 }, { /*2*/ });"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}

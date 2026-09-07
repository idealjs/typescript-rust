use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_combine_overloads_return_type() {
    let content = r#"interface A { a: number }
interface B { b: number }
declare function f(n: number): A;
declare function f(s: string): B;
f()./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

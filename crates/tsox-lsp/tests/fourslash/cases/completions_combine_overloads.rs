use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_combine_overloads() {
    let content = r#"interface A { a: number }
interface B { b: number }
declare function f(a: A): void;
declare function f(b: B): void;
f({ /**/ });"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

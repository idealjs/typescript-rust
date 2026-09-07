use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_info_with_explicit_type_arguments() {
    let content = r#"interface I { x: number; y: number; }

declare function f<T>(x: T, y: number): void;
f<I>({ /*f*/ });

declare function g<T>(x: keyof T, y: number): void;
g<I>("[|/*g*/|]");"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "f", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "g", &fourslash.CompletionsExpectedList{
}

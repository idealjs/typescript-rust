use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_keyof() {
    let content = r#"interface A { a: number; };
interface B { a: number; b: number; };
function f<T extends keyof A>(key: T) {}
f("[|/*f*/|]");
function g<T extends keyof B>(key: T) {}
g("[|/*g*/|]");"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "f", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "g", &fourslash.CompletionsExpectedList{
}

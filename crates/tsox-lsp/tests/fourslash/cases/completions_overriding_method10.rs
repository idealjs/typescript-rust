use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_overriding_method10() {
    let content = r#"// @Filename: a.ts
// @newline: LF
interface Base {
    a: string;
    b(a: string): void;
    c(a: string): string;
    c(a: number): number;
}
class Sub implements Base {
   /*a*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}

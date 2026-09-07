use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_overriding_method3() {
    let content = r#"// @newline: LF
// @Filename: boo.d.ts
interface Ghost {
    boo(): string;
}

declare class Poltergeist implements Ghost {
    /*b*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
}

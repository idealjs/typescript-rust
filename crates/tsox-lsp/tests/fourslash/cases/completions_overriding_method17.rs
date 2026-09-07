use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_overriding_method17() {
    let content = r#"// @Filename: a.ts
// @newline: LF
interface Interface {
    method(): void;
}

export class Class implements Interface {
    property = "yadda";

    /**/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

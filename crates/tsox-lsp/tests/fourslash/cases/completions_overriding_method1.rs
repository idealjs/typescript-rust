use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_overriding_method1() {
    let content = r#"// @newline: LF
// @Filename: h.ts
// @noImplicitOverride: true
class HBase {
    foo(a: string): void {}
}

class HSub extends HBase {
    [|f/*h*/|]
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "h", &fourslash.CompletionsExpectedList{
}

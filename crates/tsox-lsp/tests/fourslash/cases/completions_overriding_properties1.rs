use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_overriding_properties1() {
    let content = r#"// @newline: LF
// @Filename: a.ts
class Base {
    protected foo: string = "bar";
}

class Sub extends Base {
    /*a*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}

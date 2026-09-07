use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_for_recursive_generic_types_member() {
    let content = r#"export class TestBase<T extends TestBase<T>>
{
    public publicMethod(p: any): void {}
    private privateMethod(p: any): void {}
    protected protectedMethod(p: any): void {}
    public test(t: T): void
    {
        t./**/
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

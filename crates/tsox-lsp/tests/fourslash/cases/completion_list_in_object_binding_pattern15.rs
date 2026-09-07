use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_object_binding_pattern15() {
    let content = r#"class Foo {
    private   xxx1 = 1;
    protected xxx2 = 2;
    public    xxx3 = 3;
    private   static xxx4 = 4;
    protected static xxx5 = 5;
    public    static xxx6 = 6;
    foo() {
        const { /*1*/ } = this;
        const { /*2*/ } = Foo;
    }
}

const { /*3*/ } = new Foo();
const { /*4*/ } = Foo;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}

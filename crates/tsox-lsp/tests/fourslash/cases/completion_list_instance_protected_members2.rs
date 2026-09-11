use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_instance_protected_members2() {
    let content = r#"class Base {
    private privateMethod() { }
    private privateProperty;

    protected protectedMethod() { }
    protected protectedProperty;

    public publicMethod() { }
    public publicProperty;

    protected protectedOverriddenMethod() { }
    protected protectedOverriddenProperty;
}

class C1 extends Base {
    protected  protectedOverriddenMethod() { }
    protected  protectedOverriddenProperty;

    test() {
        this./*1*/;
        super./*2*/;

        var b: Base;
        var c: C1;

        b./*3*/;
        c./*4*/;
    }
}"#;
    let mut s = Session::new_for_test("completionListInstanceProtectedMembers2", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}

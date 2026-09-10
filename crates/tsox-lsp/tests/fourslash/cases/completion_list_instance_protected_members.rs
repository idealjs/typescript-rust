use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_instance_protected_members() {
    let content = r#"class Base {
    private privateMethod() { }
    private privateProperty;

    protected protectedMethod() { }
    protected protectedProperty;

    public publicMethod() { }
    public publicProperty;

    protected protectedOverriddenMethod() { }
    protected protectedOverriddenProperty;

    test() {
        this./*1*/;

        var b: Base;
        var c: C1;

        b./*2*/;
        c./*3*/;
    }
}

class C1 extends Base {
    protected  protectedOverriddenMethod() { }
    protected  protectedOverriddenProperty;
}"#;
    let mut s = Session::new_for_test("completionListInstanceProtectedMembers", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["privateMethod", "privateProperty", "protectedMethod", "protectedProperty", "publicMethod", "publicProperty", "test"]);
}

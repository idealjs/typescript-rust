use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_instance_protected_members3() {
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
}

 var b: Base;
 var c: C1;
 b./*1*/;
 c./*2*/;"#;
    let mut s = Session::new_for_test("completionListInstanceProtectedMembers3", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}

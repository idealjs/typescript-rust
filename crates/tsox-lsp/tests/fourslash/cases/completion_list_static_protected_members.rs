use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_static_protected_members() {
    let content = r#"class Base {
    private static privateMethod() { }
    private static privateProperty;

    protected static protectedMethod() { }
    protected static protectedProperty;

    public static publicMethod() { }
    public static publicProperty;

    protected static protectedOverriddenMethod() { }
    protected static protectedOverriddenProperty;

    static test() {
        Base./*1*/;
        this./*2*/;
        C1./*3*/;
    }
}

class C1 extends Base {
    protected static protectedOverriddenMethod() { }
    protected static protectedOverriddenProperty;
}"#;
    let mut s = Session::new_for_test("completionListStaticProtectedMembers", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}

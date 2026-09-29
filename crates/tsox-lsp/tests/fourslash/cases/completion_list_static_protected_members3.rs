use tsox_lsp::fourslash::Session;


#[test]
fn completion_list_static_protected_members3() {
    let content = r#"// @lib: es5
class Base {
    private static privateMethod() { }
    private static privateProperty;

    protected static protectedMethod() { }
    protected static protectedProperty;

    public static publicMethod() { }
    public static publicProperty;

    protected static protectedOverriddenMethod() { }
    protected static protectedOverriddenProperty;
}

class C3 extends Base {
    protected static protectedOverriddenMethod() { }
    protected static protectedOverriddenProperty;
}

Base./*1*/;
C3./*2*/;"#;
    let _s = Session::new_for_test("completionListStaticProtectedMembers3", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}

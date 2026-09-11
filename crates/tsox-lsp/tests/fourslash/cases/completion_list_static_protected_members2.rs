use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_static_protected_members2() {
    let content = r#"// @target: es2015
// @lib: es5
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

class C2 extends Base {
    protected static protectedOverriddenMethod() { }
    protected static protectedOverriddenProperty;

    static test() {
        Base./*1*/;
        C2./*2*/;
        this./*3*/;
        super./*4*/;
    }
}"#;
    let mut s = Session::new_for_test("completionListStaticProtectedMembers2", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2", "3"}, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}

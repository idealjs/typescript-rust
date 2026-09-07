use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_class_members() {
    let content = r#"// @lib: es5
class Class {
    private privateInstanceMethod() { }
    public publicInstanceMethod() { }

    private privateProperty = 1;
    public publicProperty = 1;

    private static privateStaticProperty = 1;
    public static publicStaticProperty = 1;

    private static privateStaticMethod() { }
    public static publicStaticMethod() {
        Class./*staticsInsideClassScope*/publicStaticMethod();
        var c = new Class();
        c./*instanceMembersInsideClassScope*/privateProperty;
    }
}

Class./*staticsOutsideClassScope*/publicStaticMethod();
var c = new Class();
c./*instanceMembersOutsideClassScope*/privateProperty;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "staticsInsideClassScope", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "instanceMembersInsideClassScope", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "staticsOutsideClassScope", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "instanceMembersOutsideClassScope", &fourslash.CompletionsExpectedList{
}
